    
    
    
mod unit {

    use mongodb::Client;
    use bson::{document::ValueAccessError, Bson, Document};
    use async_std::task;

    async fn simple() {
        let client = Client::with_uri_str("mongodb://example.com").await;
    
        for i in 0..5 {
            let client_ref = client.clone().unwrap();
    
            task::spawn(async move {
                let collection = client_ref.database("items").collection::<Document>(&format!("coll{}", i));
                println!("{:?}", "hi");
    
                // Do something with the collection
            });
        }
    }

    #[test]
    fn test_simple() {
        simple();
        
        assert!(false)
    }
}