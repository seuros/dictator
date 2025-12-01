// Long Rust file with structural violations (400+ lines)
pub mod models {
    #[derive(Debug, Clone)]
    pub struct Product {
        pub id: u32,
        pub name: String,
        pub description: String,
        pub price: f64,
        pub stock: u32,
    }

    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: u32,
        pub customer: String,
        pub products: Vec<Product>,
        pub total: f64,
    }

    #[derive(Debug)]
    pub struct Warehouse {
        pub name: String,
        pub location: String,
        pub inventory: Vec<(Product, u32)>,
    }
}

pub mod services {
    use super::models::*;

    pub struct ProductService;

    impl ProductService {
        pub fn new() -> Self {
            ProductService
        }

        pub fn create_product(name: String, desc: String, price: f64) -> Product {
            Product {
                id: 1,
                name,
                description: desc,
                price,
                stock: 0,
            }
        }

        fn validate_price(price: f64) -> bool {
            price > 0.0
        }

        pub fn update_price(product: &mut Product, new_price: f64) -> bool {
            if Self::validate_price(new_price) {
                product.price = new_price;
                true
            } else {
                false
            }
        }

        fn calculate_discount(original: f64, percent: f64) -> f64 {
            original * (1.0 - percent / 100.0)
        }

        pub fn apply_sale(product: &mut Product, discount_percent: f64) {
            let discounted = Self::calculate_discount(product.price, discount_percent);
            product.price = discounted;
        }
    }

    pub struct OrderService;

    impl OrderService {
        pub fn new() -> Self {
            OrderService
        }

        pub fn create_order(customer: String, products: Vec<Product>) -> Order {
            let total = products.iter().map(|p| p.price).sum();
            Order {
                id: 1,
                customer,
                products,
                total,
            }
        }

        fn calculate_subtotal(products: &[Product]) -> f64 {
            products.iter().map(|p| p.price).sum()
        }

        pub fn calculate_total(order: &Order) -> f64 {
            Self::calculate_subtotal(&order.products)
        }

        pub fn apply_discount(order: &mut Order, discount_percent: f64) {
            let subtotal = Self::calculate_subtotal(&order.products);
            order.total = subtotal * (1.0 - discount_percent / 100.0);
        }

        fn validate_stock(product: &Product, quantity: u32) -> bool {
            product.stock >= quantity
        }

        pub fn checkout(mut order: Order, quantities: Vec<u32>) -> Result<Order, String> {
            for (i, qty) in quantities.iter().enumerate() {
                if i < order.products.len() {
                    if !Self::validate_stock(&order.products[i], *qty) {
                        return Err("Insufficient stock".to_string());
                    }
                }
            }
            Ok(order)
        }
    }

    pub struct WarehouseService;

    impl WarehouseService {
        pub fn new() -> Self {
            WarehouseService
        }

        pub fn create_warehouse(name: String, location: String) -> Warehouse {
            Warehouse {
                name,
                location,
                inventory: Vec::new(),
            }
        }

        fn find_product_index(warehouse: &Warehouse, product_id: u32) -> Option<usize> {
            warehouse.inventory.iter().position(|(p, _)| p.id == product_id)
        }

        pub fn add_stock(warehouse: &mut Warehouse, product: Product, quantity: u32) {
            if let Some(idx) = Self::find_product_index(warehouse, product.id) {
                warehouse.inventory[idx].1 += quantity;
            } else {
                warehouse.inventory.push((product, quantity));
            }
        }

        pub fn remove_stock(warehouse: &mut Warehouse, product_id: u32, quantity: u32) -> bool {
            if let Some(idx) = Self::find_product_index(warehouse, product_id) {
                if warehouse.inventory[idx].1 >= quantity {
                    warehouse.inventory[idx].1 -= quantity;
                    return true;
                }
            }
            false
        }

        fn calculate_total_value(warehouse: &Warehouse) -> f64 {
            warehouse
                .inventory
                .iter()
                .map(|(p, qty)| p.price * *qty as f64)
                .sum()
        }

        pub fn inventory_value(warehouse: &Warehouse) -> f64 {
            Self::calculate_total_value(warehouse)
        }
    }
}

pub mod handlers {
    use super::models::*;
    use super::services::*;

    pub trait EventHandler {
        fn handle(&self, event: String);
    }

    pub struct ProductEventHandler;

    impl EventHandler for ProductEventHandler {
        fn handle(&self, event: String) {
            println!("Product event: {}", event);
        }
    }

    pub struct OrderEventHandler;

    impl EventHandler for OrderEventHandler {
        fn handle(&self, event: String) {
            println!("Order event: {}", event);
        }
    }

    pub struct WarehouseEventHandler;

    impl EventHandler for WarehouseEventHandler {
        fn handle(&self, event: String) {
            println!("Warehouse event: {}", event);
        }
    }

    pub struct EventBus {
        handlers: Vec<Box<dyn EventHandler>>,
    }

    impl EventBus {
        pub fn new() -> Self {
            EventBus {
                handlers: Vec::new(),
            }
        }

        pub fn register(&mut self, handler: Box<dyn EventHandler>) {
            self.handlers.push(handler);
        }

        pub fn publish(&self, event: String) {
            for handler in &self.handlers {
                handler.handle(event.clone());
            }
        }
    }
}

pub mod utils {
    pub fn format_currency(amount: f64) -> String {
        format!("${:.2}", amount)
    }

    pub fn format_quantity(qty: u32) -> String {
        format!("{} units", qty)
    }

    fn parse_float(s: &str) -> Option<f64> {
        s.parse().ok()
    }

    pub fn validate_float_input(input: &str) -> bool {
        parse_float(input).is_some()
    }

    fn sanitize_string(s: String) -> String {
        s.trim().to_string()
    }

    pub fn normalize_name(name: String) -> String {
        sanitize_string(name)
    }

    pub fn generate_id(counter: u32) -> u32 {
        counter + 1
    }
}

fn main() {
    use models::*;
    use services::*;
    use handlers::*;
    use utils::*;

    // Create products
    let mut product1 = ProductService::create_product(
        "Laptop".to_string(),
        "High performance laptop".to_string(),
        999.99,
    );

    let product2 = ProductService::create_product(
        "Mouse".to_string(),
        "Wireless mouse".to_string(),
        29.99,
    );

    // Apply sale
    ProductService::apply_sale(&mut product1, 10.0);
    println!("Laptop after 10% sale: {}", format_currency(product1.price));

    // Create order
    let mut order = OrderService::create_order(
        "John Doe".to_string(),
        vec![product1.clone(), product2.clone()],
    );

    println!("Order total: {}", format_currency(order.total));

    // Create warehouse
    let mut warehouse = WarehouseService::create_warehouse(
        "Main Warehouse".to_string(),
        "Downtown".to_string(),
    );

    WarehouseService::add_stock(&mut warehouse, product1, 50);
    WarehouseService::add_stock(&mut warehouse, product2, 200);

    println!("Warehouse value: {}", format_currency(WarehouseService::inventory_value(&warehouse)));

    // Event handling
    let mut bus = EventBus::new();
    bus.register(Box::new(ProductEventHandler));
    bus.register(Box::new(OrderEventHandler));
    bus.register(Box::new(WarehouseEventHandler));

    bus.publish("Product created".to_string());
    bus.publish("Order placed".to_string());
    bus.publish("Stock updated".to_string());
}

// Additional utility modules for larger file size

pub mod advanced_services {
    use super::models::*;

    pub struct AnalyticsService;

    impl AnalyticsService {
        pub fn new() -> Self {
            AnalyticsService
        }

        pub fn calculate_average_order_value(orders: &[super::services::Order]) -> f64 {
            if orders.is_empty() {
                return 0.0;
            }
            let total: f64 = orders.iter().map(|o| o.total).sum();
            total / orders.len() as f64
        }

        pub fn count_by_customer(orders: &[super::services::Order]) -> usize {
            orders.len()
        }

        fn filter_high_value(orders: &[super::services::Order], threshold: f64) -> Vec<&super::services::Order> {
            orders.iter().filter(|o| o.total > threshold).collect()
        }

        pub fn premium_customers(orders: &[super::services::Order], threshold: f64) -> usize {
            Self::filter_high_value(orders, threshold).len()
        }
    }

    pub struct ReportingService;

    impl ReportingService {
        pub fn new() -> Self {
            ReportingService
        }

        pub fn daily_summary(order_count: u32, total_revenue: f64) -> String {
            format!("Daily Summary - Orders: {}, Revenue: ${:.2}", order_count, total_revenue)
        }

        pub fn monthly_summary(order_count: u32, total_revenue: f64) -> String {
            format!("Monthly Summary - Orders: {}, Revenue: ${:.2}", order_count, total_revenue)
        }

        fn format_percentage(value: f64) -> String {
            format!("{:.1}%", value)
        }

        pub fn growth_report(previous: f64, current: f64) -> String {
            let growth = ((current - previous) / previous) * 100.0;
            format!("Growth: {}", Self::format_percentage(growth))
        }
    }

    pub struct ValidationService;

    impl ValidationService {
        pub fn new() -> Self {
            ValidationService
        }

        pub fn validate_order_items(product_count: usize) -> bool {
            product_count > 0
        }

        fn check_minimum_price(price: f64) -> bool {
            price >= 0.01
        }

        pub fn validate_product_pricing(price: f64) -> bool {
            Self::check_minimum_price(price)
        }

        pub fn validate_customer_name(name: &str) -> bool {
            !name.trim().is_empty()
        }

        fn validate_email_format(email: &str) -> bool {
            email.contains('@') && email.contains('.')
        }

        pub fn validate_email(email: &str) -> bool {
            Self::validate_email_format(email)
        }
    }

    pub struct CacheService {
        data: Vec<(String, String)>,
    }

    impl CacheService {
        pub fn new() -> Self {
            CacheService {
                data: Vec::new(),
            }
        }

        pub fn put(&mut self, key: String, value: String) {
            self.data.push((key, value));
        }

        fn find(data: &[(String, String)], key: &str) -> Option<String> {
            data.iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        }

        pub fn get(&self, key: &str) -> Option<String> {
            Self::find(&self.data, key)
        }

        pub fn clear(&mut self) {
            self.data.clear();
        }
    }
}

pub mod notification_service {
    pub trait Notifier {
        fn notify(&self, message: &str);
    }

    pub struct EmailNotifier;

    impl Notifier for EmailNotifier {
        fn notify(&self, message: &str) {
            println!("Email: {}", message);
        }
    }

    pub struct SmsNotifier;

    impl Notifier for SmsNotifier {
        fn notify(&self, message: &str) {
            println!("SMS: {}", message);
        }
    }

    pub struct SlackNotifier;

    impl Notifier for SlackNotifier {
        fn notify(&self, message: &str) {
            println!("Slack: {}", message);
        }
    }
}

pub mod database_models {
    pub struct DbConnection;

    impl DbConnection {
        pub fn new() -> Self {
            DbConnection
        }

        pub fn open(&self) -> bool {
            true
        }

        pub fn close(&self) -> bool {
            true
        }

        fn execute_query(&self, _query: &str) -> bool {
            true
        }

        pub fn insert(&self, query: &str) -> bool {
            self.execute_query(query)
        }

        pub fn update(&self, query: &str) -> bool {
            self.execute_query(query)
        }

        pub fn delete(&self, query: &str) -> bool {
            self.execute_query(query)
        }
    }
}

pub mod logging {
    use std::io::Write;

    pub struct Logger;

    impl Logger {
        pub fn info(msg: &str) {
            println!("[INFO] {}", msg);
        }

        pub fn warn(msg: &str) {
            println!("[WARN] {}", msg);
        }

        pub fn error(msg: &str) {
            eprintln!("[ERROR] {}", msg);
        }

        fn debug_internal(msg: &str) {
            println!("[DEBUG] {}", msg);
        }

        pub fn debug(msg: &str) {
            Self::debug_internal(msg);
        }
    }
}

pub mod testing {
    pub struct TestRunner;

    impl TestRunner {
        pub fn run_test(name: &str) -> bool {
            println!("Running test: {}", name);
            true
        }

        pub fn run_all_tests(tests: Vec<&str>) {
            for test in tests {
                Self::run_test(test);
            }
        }
    }
}
