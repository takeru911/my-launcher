/// 検索可能なアイテムのトレイト
pub trait Searchable {
    /// 検索対象のフィールドを取得
    fn search_fields(&self) -> Vec<(&str, &str)>; // (field_name, value)
}

/// 汎用的な検索フィルタ
pub struct SearchFilter {
    query: String,
    target_fields: Vec<String>,
}

impl SearchFilter {
    /// 新しい検索フィルタを作成（すべてのフィールドを対象）
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into().to_lowercase(),
            target_fields: Vec::new(), // 空の場合はすべてのフィールドを検索
        }
    }
    
    /// 特定のフィールドのみを検索対象にする
    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.target_fields = fields;
        self
    }
    
    /// アイテムが検索条件にマッチするかチェック
    pub fn matches<T: Searchable>(&self, item: &T) -> bool {
        // 空のクエリの場合はすべてマッチ
        if self.query.is_empty() {
            return true;
        }
        
        let fields = item.search_fields();
        
        // 検索対象フィールドが指定されている場合はそれらのみチェック
        if !self.target_fields.is_empty() {
            fields.iter().any(|(field_name, value)| {
                self.target_fields.contains(&field_name.to_string())
                    && value.to_lowercase().contains(&self.query)
            })
        } else {
            // すべてのフィールドを検索
            fields.iter().any(|(_, value)| {
                value.to_lowercase().contains(&self.query)
            })
        }
    }
}

/// 検索フィルタを使用してアイテムをフィルタリング
pub fn search_items<T: Searchable>(items: Vec<T>, filter: &SearchFilter) -> Vec<T> {
    items.into_iter()
        .filter(|item| filter.matches(item))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用の構造体
    #[derive(Debug, PartialEq)]
    struct TestItem {
        name: String,
        description: String,
        category: String,
    }

    impl Searchable for TestItem {
        fn search_fields(&self) -> Vec<(&str, &str)> {
            vec![
                ("name", &self.name),
                ("description", &self.description),
                ("category", &self.category),
            ]
        }
    }

    #[test]
    fn test_search_filter_empty_query() {
        let filter = SearchFilter::new("");
        let item = TestItem {
            name: "Test".to_string(),
            description: "Description".to_string(),
            category: "Category".to_string(),
        };
        
        // 空のクエリはすべてマッチ
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_search_filter_all_fields() {
        let filter = SearchFilter::new("test");
        
        // nameフィールドにマッチ
        let item1 = TestItem {
            name: "Test Item".to_string(),
            description: "Something".to_string(),
            category: "Other".to_string(),
        };
        assert!(filter.matches(&item1));
        
        // descriptionフィールドにマッチ
        let item2 = TestItem {
            name: "Item".to_string(),
            description: "This is a test".to_string(),
            category: "Other".to_string(),
        };
        assert!(filter.matches(&item2));
        
        // どのフィールドにもマッチしない
        let item3 = TestItem {
            name: "Item".to_string(),
            description: "Description".to_string(),
            category: "Category".to_string(),
        };
        assert!(!filter.matches(&item3));
    }

    #[test]
    fn test_search_filter_specific_fields() {
        let filter = SearchFilter::new("test")
            .with_fields(vec!["name".to_string()]);
        
        // nameフィールドにマッチ
        let item1 = TestItem {
            name: "Test Item".to_string(),
            description: "Something".to_string(),
            category: "Other".to_string(),
        };
        assert!(filter.matches(&item1));
        
        // descriptionにtestがあるが、nameフィールドのみ検索対象なのでマッチしない
        let item2 = TestItem {
            name: "Item".to_string(),
            description: "This is a test".to_string(),
            category: "Other".to_string(),
        };
        assert!(!filter.matches(&item2));
    }

    #[test]
    fn test_search_filter_case_insensitive() {
        let filter = SearchFilter::new("TEST");
        
        let item = TestItem {
            name: "test item".to_string(),
            description: "Description".to_string(),
            category: "Category".to_string(),
        };
        
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_search_items_function() {
        let items = vec![
            TestItem {
                name: "First Test".to_string(),
                description: "Description 1".to_string(),
                category: "A".to_string(),
            },
            TestItem {
                name: "Second".to_string(),
                description: "Test Description".to_string(),
                category: "B".to_string(),
            },
            TestItem {
                name: "Third".to_string(),
                description: "Description 3".to_string(),
                category: "C".to_string(),
            },
        ];
        
        let filter = SearchFilter::new("test");
        let filtered = search_items(items, &filter);
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "First Test");
        assert_eq!(filtered[1].name, "Second");
    }
}