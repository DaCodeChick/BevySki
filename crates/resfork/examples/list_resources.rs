//! Example: List resources in a MacSki resource fork file

use resfork::ResourceFork;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/home/admin/Downloads/MacSki/MacSki Color Art.rsrc";

    println!("Opening resource fork: {}", path);
    let fork = ResourceFork::open(path)?;

    println!("\nTotal resources: {}", fork.resource_count());
    println!("\nResource types found:");

    for res_type in fork.resource_types() {
        if let Some(resources) = fork.get_resources_by_type(res_type) {
            println!("  {} - {} resource(s)", res_type, resources.len());

            // Show first few resource IDs
            let ids: Vec<_> = resources.keys().take(5).collect();
            println!(
                "    IDs: {:?}{}",
                ids,
                if resources.len() > 5 { "..." } else { "" }
            );
        }
    }

    Ok(())
}
