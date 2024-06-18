use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Write},
    io::Read,
    time::Duration,
};

use itertools::Itertools;
use rand::{thread_rng, Rng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use anyhow::{anyhow, Result};

use crate::{
    db::Db,
    generator::{
        acknowledge::AcknowledgedUsizeCounter,
        constant::ConstantUsizeGenerator,
        counter::UsizeCounter,
        discrete::{Choice, DiscreteGenerator},
        sequential::SequentialUsizeGenerator,
        uniform::UniformUsizeGenerator,
        AcknowledgedCounter, Counter, Generator, NumberGenerator,
    },
    utils::{fnvhash64, RandomBytes, Value},
};

/// Operations available for a database.
pub enum Operation {
    /// Read operation.
    Read,
    /// Update operation.
    Update,
    /// Insert operation.
    Insert,
    /// Scan operation.
    Scan,
    /// Delete operation.
    Delete,
}

/// Internal operations.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum Op {
    Read,
    Update,
    Insert,
    Scan,
    ReadModifyWrite,
}

/// One experiment scenario. One object of this type will
/// be instantiated and shared among all client threads.
///
/// If you implement this class, you should support the "insertstart" property. This
/// allows the Client to proceed from multiple clients on different machines, in case
/// the client is the bottleneck. For example, if we want to load 1 million records from
/// 2 machines, the first machine should have insertstart=0 and the second insertstart=500000. Additionally,
/// the "insertcount" property, which is interpreted by Client, can be used to tell each instance of the
/// client how many inserts to do. In the example above, both clients should have insertcount=500000.
pub trait Workload {
    /// The configuration type for the workload.
    type Config: Serialize + DeserializeOwned + Debug + Clone;

    /// Create a new workload with the given config.
    fn new(config: Self::Config) -> Result<Self>
    where
        Self: Sized;
}

/// Configuration for the [`CoreWorkload`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreWorkloadConfig {
    /// The name of the database table to run queries against.
    #[serde(default = "default::table")]
    table: String,
    /// The number of fields in a record.
    #[serde(default = "default::fields")]
    fields: usize,
    /// Field name prefix.
    #[serde(default = "default::field_name_prefix")]
    field_name_prefix: String,
    /// Field length distribution.
    ///
    /// Options are "constant", "uniform", "zipfian", and "histogram".
    ///
    /// If "constant", only the `max_field_length` will be used.
    #[serde(default = "default::field_length_distribution")]
    field_length_distribution: String,
    /// Min field length.
    #[serde(default = "default::min_field_length")]
    min_field_length: usize,
    /// Max field length.
    #[serde(default = "default::max_field_length")]
    max_field_length: usize,
    /// The filename containing a field length histogram.
    ///
    /// Only used if field length distribution is "histogram".
    #[serde(default = "default::field_length_histogram_file")]
    field_length_histogram_file: String,
    /// The number of records to load into the database initially.
    #[serde(default = "default::record_count")]
    record_count: usize,
    /// The distribution of requests across the keyspace.
    ///
    /// Options are "uniform", "zipfian" and "sequential".
    #[serde(default = "default::request_distribution")]
    request_distribution: String,
    /// The scan length distribution.
    ///
    /// Options are "uniform" and "zipfian"
    #[serde(default = "default::scan_length_distribution")]
    scan_length_distribution: String,
    /// The min scan length (number of records).
    #[serde(default = "default::min_scan_length")]
    min_scan_length: usize,
    /// The max scan length (number of records).
    #[serde(default = "default::max_scan_length")]
    max_scan_length: usize,
    /// The `insert_start` property allows the client to proceed from multiple clients on different machines, in case the client is
    /// the bottleneck.
    ///
    /// For example, if we want to load 1 million records from 2 machines, the first machine should have insertstart=0
    /// and the second insertstart=500000.
    ///
    /// Additionally, the "insert_count" property, which is interpreted by client, can be used to tell each instance of
    /// the client how many inserts to do. In the example above, both clients should have insert_count as 500000.
    #[serde(default = "default::insert_start")]
    insert_start: usize,
    /// Adding zero padding to record numbers in order to match string sort order.
    /// Controls the number of 0s to left pad with.
    #[serde(default = "default::zero_padding")]
    zero_padding: usize,
    /// Deciding whether to read one field (false) or all fields (true) of a record.
    #[serde(default = "default::read_all_fields")]
    read_all_fields: bool,
    /// The name of the property for determining how to read all the fields when `read_all_fields` is `true`.
    ///
    /// If set to `true`, all the field names will be passed into the underlying client. If set to `false`,
    /// null will be passed into the underlying client. When passed a null, some clients may retrieve
    /// the entire row with a wildcard, which may be slower than naming all the fields.
    #[serde(default = "default::read_all_fields_by_name")]
    read_all_fields_by_name: bool,
    /// Deciding whether to write one field (false) or all fields (true) of a record.
    #[serde(default = "default::write_all_fields")]
    write_all_fields: bool,
    /// Deciding whether to check all returned data against the formation template to ensure data integrity.
    #[serde(default = "default::data_integrity")]
    data_integrity: bool,
    /// The order to insert records. Options are "ordered" or "hashed".
    #[serde(default = "default::insert_order")]
    insert_order: String,
    /// Proportion of transactions that are reads.
    #[serde(default = "default::read_proportion")]
    read_proportion: f64,
    /// Proportion of transactions that are updates.
    #[serde(default = "default::update_proportion")]
    update_proportion: f64,
    /// Proportion of transactions that are inserts.
    #[serde(default = "default::insert_proportion")]
    insert_proportion: f64,
    /// Proportion of transactions that are scans.
    #[serde(default = "default::scan_proportion")]
    scan_proportion: f64,
    /// Proportion of transactions that are read-modify-writes.
    #[serde(default = "default::read_modify_write_proportion")]
    read_modify_write_proportion: f64,
    /// How many times to retry when insertion of a single item to a DB fails.
    #[serde(default = "default::insertion_retry_limit")]
    insertion_retry_limit: usize,
    /// On average, how long to wait between the retries, in seconds.
    #[serde(default = "default::insertion_retry_interval")]
    insertion_retry_interval: usize,
}

/// The core benchmark scenario. Represents a set of clients doing simple CRUD operations. The
/// relative proportion of different kinds of operations, and other properties of the workload,
/// are controlled by parameters specified at runtime.
///
/// Properties to control the client:
///
/// - **fieldcount**: the number of fields in a record (default: 10)
/// - **fieldlength**: the size of each field (default: 100)
/// - **minfieldlength**: the minimum size of each field (default: 1)
/// - **readallfields**: should reads read all fields (true) or just one (false) (default: true)
/// - **writeallfields**: should updates and read/modify/writes update all fields (true) or just
/// one (false) (default: false)
/// - **readproportion**: what proportion of operations should be reads (default: 0.95)
/// - **updateproportion**: what proportion of operations should be updates (default: 0.05)
/// - **insertproportion**: what proportion of operations should be inserts (default: 0)
/// - **scanproportion**: what proportion of operations should be scans (default: 0)
/// - **readmodifywriteproportion**: what proportion of operations should be read a record,
/// modify it, write it back (default: 0)
/// - **requestdistribution**: what distribution should be used to select the records to operate
/// on - uniform, zipfian, hotspot, sequential, exponential or latest (default: uniform)
/// - **minscanlength**: for scans, what is the minimum number of records to scan (default: 1)
/// - **maxscanlength**: for scans, what is the maximum number of records to scan (default: 1000)
/// - **scanlengthdistribution**: for scans, what distribution should be used to choose the
/// number of records to scan, for each scan, between 1 and maxscanlength (default: uniform)
/// - **insertstart**: for parallel loads and runs, defines the starting record for this
/// YCSB instance (default: 0)
/// - **insertcount**: for parallel loads and runs, defines the number of records for this
/// YCSB instance (default: recordcount)
/// - **zeropadding**: for generating a record sequence compatible with string sort order by
/// 0 padding the record number. Controls the number of 0s to use for padding. (default: 1)
/// For example for row 5, with zeropadding=1 you get 'user5' key and with zeropading=8 you get
/// 'user00000005' key. In order to see its impact, zeropadding needs to be bigger than number of
/// digits in the record number.
/// - **insertorder**: should records be inserted in order by key ("ordered"), or in hashed
/// order ("hashed") (default: hashed)
/// - **fieldnameprefix**: what should be a prefix for field names, the shorter may decrease the
/// required storage size (default: "field")
pub struct CoreWorkload {
    table: String,
    field_names: Vec<String>,
    field_length_generator: Box<dyn NumberGenerator<Output = usize>>,
    operation_chooser: DiscreteGenerator<Op>,
    key_sequencer: UsizeCounter,
    ordered_inserts: bool,
    zero_padding: usize,
    data_inategrity: bool,
    insertion_retry_limit: usize,
    insertion_retry_interval: usize,
    read_all_fields: bool,
    read_all_fields_by_name: bool,
    write_all_fields: bool,
    field_chooser: UniformUsizeGenerator,
    transaction_insert_key_sequencer: AcknowledgedUsizeCounter,
    key_chooser: Box<dyn NumberGenerator<Output = usize>>,
    scan_length_generator: Box<dyn NumberGenerator<Output = usize>>,
}

impl Workload for CoreWorkload {
    type Config = CoreWorkloadConfig;

    fn new(config: Self::Config) -> Result<Self>
    where
        Self: Sized,
    {
        let field_length_generator: Box<dyn NumberGenerator<Output = usize>> =
            match config.field_length_distribution.as_str() {
                "constant" => Box::new(ConstantUsizeGenerator::new(config.max_field_length)),
                "uniform" => Box::new(UniformUsizeGenerator::new(
                    config.min_field_length,
                    config.max_field_length,
                )),
                "zipfian" => unimplemented!(),
                "histogram" => unimplemented!(),
                x => panic!("field length distribution not support: {x}"),
            };

        let scan_length_generator: Box<dyn NumberGenerator<Output = usize>> =
            match config.scan_length_distribution.as_str() {
                "uniform" => Box::new(UniformUsizeGenerator::new(
                    config.min_scan_length,
                    config.max_scan_length,
                )),
                "zipfian" => unimplemented!(),
                x => panic!("scan length distribution not support: {x}"),
            };

        let record_count = if config.record_count == 0 {
            usize::MAX
        } else {
            config.record_count
        };

        let insert_start = config.insert_start;
        let insert_count = record_count - insert_start;
        if record_count < insert_start + insert_count {
            panic!("invalid combination of insert_start ({insert_start}), insert_count ({insert_count}) and record_count ({record_count}): record_count must be equal to or larger than insert_start + insert_count")
        }

        let key_sequencer = UsizeCounter::new(insert_start);

        if config.data_integrity && config.field_length_distribution.as_str() != "constant" {
            panic!("must have constant field length to check data integrity");
        }

        let mut choices = vec![];
        if config.read_proportion > 0.0 {
            choices.push(Choice {
                val: Op::Read,
                weight: config.read_proportion,
            });
        }
        if config.update_proportion > 0.0 {
            choices.push(Choice {
                val: Op::Update,
                weight: config.update_proportion,
            });
        }
        if config.insert_proportion > 0.0 {
            choices.push(Choice {
                val: Op::Insert,
                weight: config.insert_proportion,
            });
        }
        if config.scan_proportion > 0.0 {
            choices.push(Choice {
                val: Op::Scan,
                weight: config.scan_proportion,
            });
        }
        if config.read_modify_write_proportion > 0.0 {
            choices.push(Choice {
                val: Op::ReadModifyWrite,
                weight: config.read_modify_write_proportion,
            });
        }
        let operation_generator = DiscreteGenerator::new(choices);

        let ordered_inserts = config.insert_order.as_str() != "hashed";

        let field_names = (0..config.fields)
            .map(|i| format!("{prefix}{i}", prefix = config.field_name_prefix))
            .collect_vec();
        let field_chooser = UniformUsizeGenerator::new(0, field_names.len() - 1);

        let transaction_insert_key_sequencer = AcknowledgedUsizeCounter::new(record_count);

        let key_chooser: Box<dyn NumberGenerator<Output = usize>> =
            match config.request_distribution.as_str() {
                "uniform" => Box::new(UniformUsizeGenerator::new(
                    insert_start,
                    insert_start + insert_count - 1,
                )),
                "zipfian" => unimplemented!(),
                "sequential" => Box::new(SequentialUsizeGenerator::new(
                    insert_start,
                    insert_start + insert_count - 1,
                )),
                x => panic!("request distribution distribution not support: {x}"),
            };

        Ok(Self {
            table: config.table,
            field_names,
            field_length_generator,
            operation_chooser: operation_generator,
            key_sequencer,
            ordered_inserts,
            zero_padding: config.zero_padding,
            data_inategrity: config.data_integrity,
            insertion_retry_limit: config.insertion_retry_limit,
            insertion_retry_interval: config.insertion_retry_interval,
            read_all_fields: config.read_all_fields,
            read_all_fields_by_name: config.read_all_fields_by_name,
            write_all_fields: config.write_all_fields,
            field_chooser,
            transaction_insert_key_sequencer,
            key_chooser,
            scan_length_generator,
        })
    }
}

impl CoreWorkload {
    /// Do one insert operation. Because it will be called concurrently from multiple client threads,
    /// this function must be thread safe. However, avoid synchronized, or the threads will block waiting
    /// for each other, and it will be difficult to reach the target throughput. Ideally, this function would
    /// have no side effects other than DB operations.
    pub fn insert(&self, db: impl Db) -> Result<()> {
        let key_num = self.key_sequencer.next();
        let db_key = self.build_key_name(key_num);
        let values = self.build_values(&db_key);

        self.retry(
            "insert",
            || db.insert(self.table.clone(), db_key.clone(), values.clone()),
            self.insertion_retry_limit,
            Duration::from_secs(self.insertion_retry_interval as _),
        )
    }

    /// Do one transaction operation. Because it will be called concurrently from multiple client
    /// threads, this function must be thread safe. However, avoid synchronized, or the threads will block waiting
    /// for each other, and it will be difficult to reach the target throughput. Ideally, this function would
    /// have no side effects other than DB operations.
    pub fn transaction(&self, db: impl Db) -> Result<()> {
        let op = self.operation_chooser.next();
        match op {
            Op::Read => self.txn_read(db),
            Op::Update => self.txn_update(db),
            Op::Insert => self.txn_insert(db),
            Op::Scan => self.txn_scan(db),
            Op::ReadModifyWrite => self.txn_read_modify_read(db),
        }
    }

    fn txn_read(&self, db: impl Db) -> Result<()> {
        let key_num = self.next_key_num();
        let key_name = self.build_key_name(key_num);

        let mut fields = HashSet::new();

        if !self.read_all_fields {
            let field_name = self.field_names[self.field_chooser.next()].clone();
            fields.insert(field_name);
        } else if self.data_inategrity || self.read_all_fields_by_name {
            fields.extend(self.field_names.iter().cloned());
        }

        let cells = db.read(self.table.clone(), key_name.clone(), fields.clone())?;
        if self.data_inategrity {
            self.verify_row(key_name.clone(), fields.clone(), cells)?;
        }
        Ok(())
    }

    fn txn_update(&self, db: impl Db) -> Result<()> {
        let key_num = self.next_key_num();
        let key_name = self.build_key_name(key_num);

        let values = if self.write_all_fields {
            self.build_values(&key_name)
        } else {
            self.build_single_value(&key_name)
        };

        db.update(self.table.clone(), key_name.clone(), values)
    }

    fn txn_insert(&self, db: impl Db) -> Result<()> {
        let key_num = self.transaction_insert_key_sequencer.next();

        let key_name = self.build_key_name(key_num);
        let values = self.build_values(&key_name);

        let res = db.insert(self.table.clone(), key_name, values);

        self.transaction_insert_key_sequencer.acknowledge(key_num);
        res
    }

    fn txn_scan(&self, db: impl Db) -> Result<()> {
        let key_num = self.transaction_insert_key_sequencer.next();

        let start_key_name = self.build_key_name(key_num);
        let len = self.scan_length_generator.next();

        let mut fields = HashSet::new();

        if !self.read_all_fields {
            fields.insert(self.field_names[self.field_chooser.next()].clone());
        }

        // TODO(MrCroxx): verify?
        db.scan(self.table.clone(), start_key_name, len, fields)?;

        Ok(())
    }

    fn txn_read_modify_read(&self, db: impl Db) -> Result<()> {
        let key_num = self.next_key_num();
        let key_name = self.build_key_name(key_num);

        let mut fields = HashSet::new();

        if !self.read_all_fields {
            let field_name = self.field_names[self.field_chooser.next()].clone();
            fields.insert(field_name);
        } else if self.data_inategrity || self.read_all_fields_by_name {
            fields.extend(self.field_names.iter().cloned());
        }

        let values = if self.write_all_fields {
            self.build_values(&key_name)
        } else {
            self.build_single_value(&key_name)
        };

        let cells = db.read(self.table.clone(), key_name.clone(), fields.clone())?;
        db.update(self.table.clone(), key_name.clone(), values)?;

        if self.data_inategrity {
            self.verify_row(key_name.clone(), fields.clone(), cells)?;
        }

        Ok(())
    }

    fn build_key_name(&self, mut key_num: usize) -> String {
        if !self.ordered_inserts {
            key_num = fnvhash64(key_num as _) as _;
        }
        format!("{key_num:0width$}", width = self.zero_padding)
    }

    fn build_single_value(&self, key: &str) -> HashMap<String, Value> {
        let mut ret = HashMap::new();

        let field_key = self.field_names[self.field_chooser.next()].clone();
        let size = self.field_length_generator.next();

        let value = if self.data_inategrity {
            self.build_deterministic_value(size, key, field_key.as_str())
                .into()
        } else {
            RandomBytes::new(size).into()
        };
        ret.insert(field_key, value);

        ret
    }

    fn build_values(&self, key: &str) -> HashMap<String, Value> {
        let mut ret = HashMap::new();

        for field_key in self.field_names.iter().cloned() {
            let size = self.field_length_generator.next();

            let value = if self.data_inategrity {
                self.build_deterministic_value(size, key, field_key.as_str())
                    .into()
            } else {
                RandomBytes::new(size).into()
            };
            ret.insert(field_key, value);
        }

        ret
    }

    fn build_deterministic_value(&self, size: usize, key: &str, field_key: &str) -> String {
        let mut ret = String::with_capacity(size);
        ret.write_str(key).unwrap();
        ret.write_char(':').unwrap();
        ret.write_str(field_key).unwrap();
        while ret.len() < size {
            ret.write_char(':').unwrap();
            let hash = ahash::RandomState::with_seed(0).hash_one(&ret);
            write!(&mut ret, "{hash}").unwrap();
        }
        ret.truncate(size);
        ret
    }

    fn verify_row(
        &self,
        key: String,
        fields: HashSet<String>,
        mut cells: HashMap<String, Value>,
    ) -> Result<()> {
        for field in fields.into_iter() {
            let Some(mut value) = cells.remove(&field) else {
                return Err(anyhow!("missing value for field {field}"));
            };
            let mut got = vec![];
            value.read_to_end(&mut got)?;
            let got = String::from_utf8(got).unwrap();
            let expected =
                self.build_deterministic_value(self.field_length_generator.next(), &key, &field);
            if got != expected {
                return Err(anyhow!(
                    "value mismitch for field {field}, got: {got}, expected: {expected}"
                ));
            }
        }
        todo!()
    }

    fn next_key_num(&self) -> usize {
        // FIXME(MrCroxx):
        //
        // if (keychooser instanceof ExponentialGenerator) {
        //   do {
        //     keynum = transactioninsertkeysequence.lastValue() - keychooser.nextValue().intValue();
        //   } while (keynum < 0);
        // } else {
        //   do {
        //     keynum = keychooser.nextValue().intValue();
        //   } while (keynum > transactioninsertkeysequence.lastValue());
        // }
        let mut key_num;
        loop {
            key_num = self.key_chooser.next();
            if key_num <= self.transaction_insert_key_sequencer.last() {
                break;
            }
        }
        key_num
    }

    fn retry<F>(&self, label: &str, f: F, limits: usize, interval: Duration) -> Result<()>
    where
        F: Fn() -> Result<()>,
    {
        for retry in 0..limits {
            match f() {
                Ok(()) => return Ok(()),
                Err(e) => tracing::warn!("{label} error: {e}"),
            }

            tracing::warn!("retrying {label}, retry times: {retry}");

            std::thread::sleep(Duration::from_secs_f64(
                interval.as_secs_f64() * thread_rng().gen_range(0.8..=1.2),
            ));
        }

        Err(anyhow!("{label} exceeds retry limits (limits)."))
    }
}

/// Default values for configurations.
#[allow(missing_docs)]
pub mod default {
    pub fn table() -> String {
        "ycsb".to_string()
    }

    pub fn fields() -> usize {
        10
    }

    pub fn field_name_prefix() -> String {
        "field".to_string()
    }

    pub fn field_length_distribution() -> String {
        "constant".to_string()
    }

    pub fn min_field_length() -> usize {
        1
    }

    pub fn max_field_length() -> usize {
        100
    }

    pub fn field_length_histogram_file() -> String {
        "hist.txt".to_string()
    }

    pub fn record_count() -> usize {
        0
    }

    pub fn request_distribution() -> String {
        "uniform".to_string()
    }

    pub fn min_scan_length() -> usize {
        1
    }

    pub fn max_scan_length() -> usize {
        1000
    }

    pub fn scan_length_distribution() -> String {
        "uniform".to_string()
    }

    pub fn insert_start() -> usize {
        0
    }

    pub fn zero_padding() -> usize {
        1
    }

    pub fn read_all_fields() -> bool {
        true
    }

    pub fn read_all_fields_by_name() -> bool {
        false
    }

    pub fn write_all_fields() -> bool {
        false
    }

    pub fn data_integrity() -> bool {
        false
    }

    pub fn insert_order() -> String {
        "hashed".to_string()
    }

    pub fn read_proportion() -> f64 {
        0.95
    }

    pub fn update_proportion() -> f64 {
        0.05
    }

    pub fn insert_proportion() -> f64 {
        0.0
    }

    pub fn scan_proportion() -> f64 {
        0.0
    }

    pub fn read_modify_write_proportion() -> f64 {
        0.0
    }

    pub fn insertion_retry_limit() -> usize {
        0
    }

    pub fn insertion_retry_interval() -> usize {
        3
    }
}
