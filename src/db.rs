//  Copyright 2024 MrCroxx
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::utils::Value;

/// A layer for accessing a database to be benchmarked. Each thread in the client
/// will be given its own instance of whatever DB class is to be used in the test.
/// This class should be constructed using a no-argument constructor, so we can
/// load it dynamically. Any argument-based initialization should be
/// done by init().
///
/// Note that YCSB does not make any use of the return codes returned by this class.
/// Instead, it keeps a count of the return values and presents them to the user.
///
/// The semantics of methods such as insert, update and delete vary from database
/// to database.  In particular, operations may or may not be durable once these
/// methods commit, and some systems may return 'success' regardless of whether
/// or not a tuple with a matching key existed before the call.  Rather than dictate
/// the exact semantics of these methods, we recommend you either implement them
/// to match the database's default semantics, or the semantics of your
/// target application.  For the sake of comparison between experiments we also
/// recommend you explain the semantics you chose when presenting performance results.
pub trait Db {
    /// Configuration type for db.
    type Config;

    /// Initialize any state for this DB.
    /// Called once per DB instance; there is one DB instance per client thread.
    fn init(&self) -> Result<()>;

    /// Cleanup any state for this DB.
    /// Called once per DB instance; there is one DB instance per client thread.
    fn cleanup(&self) -> Result<()>;

    /// Insert a record in the database. Any field/value pairs in the specified values HashMap will be written into the
    /// record with the specified record key.
    ///
    /// * `table` - The name of the table
    /// * `key` - The record key of the record to insert.
    /// * `values` - A HashMap of field/value pairs to insert in the record
    ///
    /// Returns the result of the operation.
    fn insert(&self, table: String, key: String, values: HashMap<String, Value>) -> Result<()>;

    /// Read a record from the database. Each field/value pair from the result will be stored in a HashMap.
    ///
    /// * `table` - The name of the table
    /// * `key` - The record key of the record to read.
    /// * `fields` - The list of fields to read, or null for all of them
    /// * `result` - A HashMap of field/value pairs for the result
    ///
    /// Returns the result of the operation.
    fn read(
        &self,
        table: String,
        key: String,
        fields: HashSet<String>,
    ) -> Result<HashMap<String, Value>>;

    /// Update a record in the database. Any field/value pairs in the specified values HashMap will be written into the
    /// record with the specified record key, overwriting any existing values with the same field name.
    ///
    /// * `table` - The name of the table
    /// * `key` - The record key of the record to write.
    /// * `values` - A HashMap of field/value pairs to update in the record
    ///
    /// Returns the result of the operation.
    fn update(&self, table: String, key: String, values: HashMap<String, Value>) -> Result<()>;

    /// Perform a range scan for a set of records in the database. Each field/value pair from the result will be stored
    /// in a HashMap.
    ///
    /// * `table` - The name of the table
    /// * `startkey` - The record key of the first record to read.
    /// * `recordcount` - The number of records to read
    /// * `fields` - The list of fields to read, or null for all of them
    /// * `result` - A Vector of HashMaps, where each HashMap is a set field/value pairs for one record
    ///
    /// Returns the result of the operation.
    fn scan(
        &self,
        table: String,
        start_key: String,
        len: usize,
        fields: HashSet<String>,
    ) -> Result<HashMap<String, Vec<Value>>>;

    /// Delete a record from the database.
    ///
    /// * `table` - The name of the table
    /// * `key` - The record key of the record to delete.
    ///
    /// Returns the result of the operation.
    fn delete(&self, table: String, key: String);
}
