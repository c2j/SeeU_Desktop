use inote::mermaid::DiagramType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_diagram_detection() {
        let code = "classDiagram\n    class Animal {\n        +String name\n    }";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::ClassDiagram);
    }

    #[test]
    fn test_state_diagram_detection() {
        let code = "stateDiagram-v2\n    [*] --> Idle";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::StateDiagram);
    }

    #[test]
    fn test_git_graph_detection() {
        let code = "gitGraph\n    commit id: \"Initial\"";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::GitGraph);
    }

    #[test]
    fn test_user_journey_detection() {
        let code = "journey\n    title My working day";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::UserJourney);
    }

    #[test]
    fn test_entity_relationship_detection() {
        let code = "erDiagram\n    CUSTOMER {\n        string name\n    }";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::EntityRelationship);
    }

    #[test]
    fn test_flowchart_detection() {
        let code = "flowchart TD\n    A --> B";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::Flowchart);
    }

    #[test]
    fn test_sequence_detection() {
        let code = "sequenceDiagram\n    participant A";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::Sequence);
    }

    #[test]
    fn test_unknown_detection() {
        let code = "unknownDiagram\n    some content";
        let diagram_type = DiagramType::from_code(code);
        assert_eq!(diagram_type, DiagramType::Unknown);
    }
}
