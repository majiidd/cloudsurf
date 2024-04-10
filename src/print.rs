use std::net::Ipv4Addr;

use prettytable::{row, Cell, Row, Table};

pub fn ips(ips: Vec<(Ipv4Addr, u128)>) {
    let mut table = Table::new();
    table.add_row(row!["", "IP Address", "Latency (ms)"]);

    let mut row_num = 1;
    for (ip, latency) in ips {
        table.add_row(Row::new(vec![
            Cell::new(&row_num.to_string()),
            Cell::new(&ip.to_string()),
            Cell::new(&latency.to_string()),
        ]));

        row_num += 1;
    }

    table.printstd();
}