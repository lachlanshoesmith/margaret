use margaret::models::database::Column;
use margaret::models::filters::{get_filter_conditions, RelationColumnFilter};
use margaret::{follow_relations, get_db_columns, query_column_values};
use std::io::{self, Write};
use std::{collections::HashMap, error::Error};
use struct_iterable::Iterable;

use clap::Parser;

use margaret::models::{
    blocks::Blocks,
    database::{fetch_notion_database, DatabaseCredentials},
    filters::{CheckboxColumnFilter, ColumnFilter, QueryFilter, RichTextColumnFilter},
};

#[derive(Parser, Debug)]
struct Args {
    notion_db: String,
    integration_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let credentials = DatabaseCredentials {
        id: args.notion_db,
        token: args.integration_secret,
    };
    let db = fetch_notion_database(&credentials).await?;

    let columns = get_db_columns(&db.body)?;

    if columns.is_none() {
        println!("Database has no columns.");
        return Ok(());
    }

    let columns: Vec<Column> = columns.unwrap();

    println!("Welcome to Margaret! ✉️ 👋\n");
    println!(
        "I found the following columns in the database {}:",
        credentials.id
    );

    for column in columns.iter() {
        println!("- {} <{}>", column.name, column.column_type);
    }

    let mut column_to_print: Option<&Column>;
    let mut filter: ColumnFilter;
    let mut filter_conditions: HashMap<String, String> = HashMap::new();

    loop {
        loop {
            print!("\nWhich column do you want to print? ");
            let mut column_to_print_name = String::new();
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut column_to_print_name).unwrap();
            let column_to_print_name = column_to_print_name.trim().to_string();

            column_to_print = columns.iter().find(|col| col.name == column_to_print_name);
            if column_to_print.is_none() {
                println!("The column '{}' does not exist.", column_to_print_name);
                continue;
            }
            break;
        }
        print!("\nWhich column do you want to query? ");
        let mut query_column_name = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut query_column_name).unwrap();
        let query_column_name = query_column_name.trim().to_string();

        let query_column = columns.iter().find(|col| col.name == query_column_name);
        if query_column.is_none() {
            println!("The column '{}' does not exist.", query_column_name);
            continue;
        }
        let query_column = query_column.unwrap();

        println!(
            "This column's of the '{}' type, so it has the following filter conditions:",
            query_column.column_type
        );

        let all_filter_conditions = get_filter_conditions();

        match query_column.column_type.as_str() {
            "checkbox" => {
                for (field_name, _field_value) in CheckboxColumnFilter::default().iter() {
                    println!(
                        "- {0:?} <{1}>",
                        field_name,
                        all_filter_conditions.get(field_name).unwrap()
                    );
                }
            }
            "rich_text" => {
                for (field_name, _field_value) in RichTextColumnFilter::default().iter() {
                    println!(
                        "- {0:?} <{1}>",
                        field_name,
                        all_filter_conditions.get(field_name).unwrap()
                    );
                }
            }
            "relation" => {
                for (field_name, _field_value) in RelationColumnFilter::default().iter() {
                    println!(
                        "- {0:?} <{1}>",
                        field_name,
                        all_filter_conditions.get(field_name).unwrap()
                    );
                }
                println!("- \"follow\" <true>");
                println!("💁 If this relation links to a database and you want to query a property from that database, choose 'follow'.")
            }
            _ => {
                println!(
                    "The column type '{}' is not supported.",
                    query_column.column_type
                );
                continue;
            }
        };

        let mut i = 0;
        loop {
            print!("\nWhich filter condition do you want to apply? ");

            if i == 1 {
                print!("\n(Leave blank to stop adding filter conditions): ");
            }

            let mut filter_condition_name = String::new();
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut filter_condition_name).unwrap();
            let filter_condition_name = filter_condition_name.trim().to_string();

            if filter_condition_name.is_empty() {
                if i == 0 {
                    println!("You need to add at least one filter condition.");
                    continue;
                }
                break;
            }

            if !all_filter_conditions.contains_key(&filter_condition_name) {
                println!(
                    "The filter condition '{}' does not exist.",
                    filter_condition_name
                );
                continue;
            }
            print!(
                "What is the value of the filter condition '{}'? ",
                filter_condition_name
            );
            let mut filter_condition_value = String::new();
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut filter_condition_value).unwrap();
            let filter_condition_value = filter_condition_value.trim().to_string();
            filter_conditions.insert(filter_condition_name.clone(), filter_condition_value);
            i += 1;
        }

        filter = ColumnFilter {
            property: query_column.name.clone(),
            ..Default::default()
        };

        match query_column.column_type.as_str() {
            "rich_text" => {
                filter.rich_text = Some(RichTextColumnFilter {
                    contains: filter_conditions.get("contains").cloned(),
                    does_not_contain: filter_conditions.get("does_not_contain").cloned(),
                    is_empty: filter_conditions
                        .get("is_empty")
                        .map(|value| value == "true")
                        .or(Some(false)),
                    is_not_empty: filter_conditions
                        .get("is_not_empty")
                        .map(|value| value == "true")
                        .or(Some(false)),
                    starts_with: filter_conditions.get("starts_with").cloned(),
                    ends_with: filter_conditions.get("ends_with").cloned(),
                    equals: filter_conditions.get("equals").cloned(),
                    does_not_equal: filter_conditions.get("does_not_equal").cloned(),
                })
            }
            "checkbox" => {
                filter.checkbox = Some(CheckboxColumnFilter {
                    equals: filter_conditions
                        .get("equals")
                        .cloned()
                        .map(|value| matches!(value.as_str(), "true")),
                    does_not_equal: filter_conditions
                        .get("does_not_equal")
                        .cloned()
                        .map(|value| matches!(value.as_str(), "true")),
                })
            }
            "relation" => {
                filter.relation = Some(RelationColumnFilter {
                    is_empty: filter_conditions
                        .get("is_empty")
                        .cloned()
                        .map(|value| matches!(value.as_str(), "true")),
                    is_not_empty: filter_conditions
                        .get("is_not_empty")
                        .cloned()
                        .map(|value| matches!(value.as_str(), "true")),
                    contains: filter_conditions.get("contains").cloned(),
                    does_not_contain: filter_conditions.get("does_not_contain").cloned(),
                });

                if filter_conditions.contains_key("follow") {
                    filter.relation.as_mut().unwrap().is_not_empty = Some(true);
                };
            }
            _ => {
                println!(
                    "The column type '{}' is not supported.",
                    query_column.column_type
                );
            }
        };

        print!("Would you like to chain this filter with another (AND/OR/n)? ");
        let mut chain = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut chain).unwrap();
        let chain = chain.trim().to_string();

        if chain == "n" {
            break;
        }
    }

    let query = QueryFilter::ColumnFilter(Box::new(filter));

    print!("\nFetching data from Notion...");
    io::stdout().flush().unwrap();
    let blocks = query_column_values(&credentials, column_to_print.unwrap(), &query).await?;

    if filter_conditions.contains_key("follow") {
        print!(
            "\rFollowing relations (this may take some time)...{}",
            " ".repeat(28)
        );
        io::stdout().flush().unwrap();
        for block in &blocks {
            if let Blocks::Relation(_) = block {
                follow_relations(&credentials.token, block).await?;
            }
        }
    }

    let emails: Vec<String> = blocks.iter().map(|block| block.to_string()).collect();
    print!("\r{}\n\n", "=".repeat(50));
    for email in emails {
        println!("'{}'", email);
    }
    Ok(())
}
