use iterminal::export_ui::ExportDialog;
use arboard::Clipboard;

fn main() {
    println!("iTerminal Clipboard Test");
    println!("========================");
    
    // Test 1: Basic clipboard functionality
    println!("\n1. Testing basic clipboard functionality...");
    match Clipboard::new() {
        Ok(mut clipboard) => {
            let test_text = "Hello from iTerminal clipboard test!";
            
            // Set text
            match clipboard.set_text(test_text.to_string()) {
                Ok(_) => {
                    println!("   ✓ Successfully set text to clipboard");
                    
                    // Get text back
                    match clipboard.get_text() {
                        Ok(retrieved_text) => {
                            if retrieved_text == test_text {
                                println!("   ✓ Successfully retrieved text from clipboard");
                                println!("   ✓ Text matches: '{}'", retrieved_text);
                            } else {
                                println!("   ✗ Text mismatch!");
                                println!("     Expected: '{}'", test_text);
                                println!("     Got: '{}'", retrieved_text);
                            }
                        }
                        Err(e) => {
                            println!("   ✗ Failed to get text from clipboard: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("   ✗ Failed to set text to clipboard: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ✗ Failed to create clipboard instance: {}", e);
        }
    }
    
    // Test 2: ExportDialog clipboard functionality
    println!("\n2. Testing ExportDialog clipboard functionality...");
    let dialog = ExportDialog::default();
    let test_content = "This is a test terminal output\nLine 2\nLine 3\n";
    
    match dialog.copy_to_clipboard(test_content) {
        Ok(_) => {
            println!("   ✓ ExportDialog clipboard copy successful");
            
            // Verify by reading back
            match Clipboard::new() {
                Ok(mut clipboard) => {
                    match clipboard.get_text() {
                        Ok(retrieved) => {
                            if retrieved == test_content {
                                println!("   ✓ Content verification successful");
                            } else {
                                println!("   ✗ Content verification failed");
                                println!("     Expected: {:?}", test_content);
                                println!("     Got: {:?}", retrieved);
                            }
                        }
                        Err(e) => {
                            println!("   ✗ Failed to verify clipboard content: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("   ✗ Failed to create clipboard for verification: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ✗ ExportDialog clipboard copy failed: {}", e);
        }
    }
    
    // Test 3: Large content test
    println!("\n3. Testing large content clipboard functionality...");
    let large_content = "Line ".repeat(1000) + "\n";
    
    match dialog.copy_to_clipboard(&large_content) {
        Ok(_) => {
            println!("   ✓ Large content clipboard copy successful");
            println!("   ✓ Content size: {} characters", large_content.len());
        }
        Err(e) => {
            println!("   ✗ Large content clipboard copy failed: {}", e);
        }
    }
    
    // Test 4: Special characters test
    println!("\n4. Testing special characters...");
    let special_content = "Special chars: 🚀 📋 ✅ ❌ 中文 العربية русский\nTabs:\t\tand\nnewlines\n";
    
    match dialog.copy_to_clipboard(special_content) {
        Ok(_) => {
            println!("   ✓ Special characters clipboard copy successful");
        }
        Err(e) => {
            println!("   ✗ Special characters clipboard copy failed: {}", e);
        }
    }
    
    println!("\nClipboard test completed!");
    println!("Note: The clipboard now contains the last test content.");
    println!("You can paste it elsewhere to verify the functionality.");
}
