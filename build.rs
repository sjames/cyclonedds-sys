fn main() {
    build::main();
}

mod build {

    extern crate bindgen;

    use std::env;
    use std::path::Path;
    use std::path::PathBuf;
    //use walkdir::{DirEntry, WalkDir};
    use glob::glob;
    
    static ENV_PREFIX: &str = "CYCLONEDDS";
    static LINKLIB: &str = "ddsc";

    pub enum HeaderLocation {
        FromCMakeEnvironment(std::vec::Vec<String>, String),
        FromEnvironment(std::vec::Vec<String>),
    }

    fn find_cyclonedds() -> Option<HeaderLocation> {
        // The library name does not change. Print that out right away
        println!("cargo:rustc-link-lib={}", LINKLIB);
        //first priority is environment variable.
        if let Ok(dir) = env::var(format!("{}_LIB_DIR", ENV_PREFIX)) {
            println!("cargo:rustc-link-search={}", dir);

            // Now find the include path
            if let Ok(dir) = env::var(format!("{}_INCLUDE_DIR", ENV_PREFIX)) {
                let path = format!("{}/dds/dds.h", &dir);
                let path = Path::new(&path);
                if path.exists() {
                    println!("Found {}", &path.to_str().unwrap());
                    let paths = vec![dir];
                    Some(HeaderLocation::FromEnvironment(paths))
                } else {
                    println!("Cannot find dds/dds.h");
                    None
                }
            } else {
                println!("LIB_DIR set but INCLUDE_DIR is unset");
                None
            }
        } 
        // now check if building using CMAKE. CycloneDDS has a cmake
        // build environment. When building within CMake, the cyclonedds need not
        // be "installed", so multiple include paths are required.
        else if let Ok(dir) = env::var("CMAKE_BINARY_DIR") {
            let cmake_bin_dir = &dir;
            let lib_dir = Path::new(&dir).join("lib");
            println!("cargo:rustc-link-search={:}", lib_dir.display());

            if let Ok(dir) = env::var("CMAKE_SOURCE_DIR") {
                println!(
                    "CMAKE_SOURCE_DIR is set to {}, searching for include path",
                    &dir
                );
                let cmake_src_dir = Path::new(&dir);
                let glob_pattern = format!("{}/**/dds/dds.h", cmake_src_dir.display());
                println!("Glob pattern: {}", &glob_pattern);
                let mut paths = std::vec::Vec::new();
                for entry in glob(&glob_pattern).expect("Glob pattern error") {
                    match entry {
                        Ok(path) => {
                            println!("{:?}", path.display());
                            let cyclone_src = path
                                .to_str()
                                .unwrap()
                                .split("cyclonedds")
                                .collect::<Vec<&str>>();
                            let mut cyclone_src = String::from(cyclone_src[0]);
                            cyclone_src.push_str("cyclonedds");

                            paths.push(format!("{}/src/core/ddsc/include", cyclone_src));
                            paths.push(format!("{}/src/core/include", cyclone_src));

                            //
                            paths.push(format!(
                                "{}/src/core/include",
                                find_cyclone_bin_dir(cmake_bin_dir).unwrap()
                            ));

                            println!("{:?}", paths);
                            break;
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }
                // now get the sysroot
                if let Ok(toolchain_sysroot) = env::var("TOOLCHAIN_SYSROOT") {
                    Some(HeaderLocation::FromCMakeEnvironment(
                        paths,
                        toolchain_sysroot,
                    ))
                } else {
                    println!("Unable to get TOOLCHAIN_SYSROOT");
                    Some(HeaderLocation::FromCMakeEnvironment(
                        paths,
                        "/".to_string(),
                    ))
                }
            } else {
                None
            }
        } else {
            println!("No CMAKE environment or CYCLONEDDS_[LIB|INCLUDE]_DIR found");
	    //try some defaults
            println!("cargo:rustc-link-search=/usr/local/lib");

            let path = format!("{}/dds/dds.h", "/usr/local/include");
            let path = Path::new(&path);
            if path.exists() {
                println!("Found {}", &path.to_str().unwrap());
                let paths = vec![String::from("/usr/local/include")];
                Some(HeaderLocation::FromEnvironment(paths))
            } else {
                println!("Cannot find dds/dds.h");
                None
            }
        }
    }

    fn find_cyclone_bin_dir(cmake_bin_dir: &str) -> Option<String> {
        Some(format!(
            "{}/sys/cyclonedds/src/ddsrt/include",
            cmake_bin_dir
        ))
    }

    fn add_whitelist(builder : bindgen::Builder ) -> bindgen::Builder {
      builder
        .whitelist_function("dds_enable")
        .whitelist_function("dds_delete")
        .whitelist_function("dds_get_publisher")
        .whitelist_function("dds_get_subscriber")
        .whitelist_function("dds_get_datareader")
        .whitelist_function("dds_get_mask")
        .whitelist_function("dds_get_instance_handle")
        .whitelist_function("dds_read_status")
        .whitelist_function("dds_take_status")
        .whitelist_function("dds_get_status_changes")
        .whitelist_function("dds_get_status_mask")
        .whitelist_function("dds_get_enabled_status")
        .whitelist_function("dds_set_status_mask")
        .whitelist_function("dds_set_enabled_status")
        .whitelist_function("dds_get_qos")
        .whitelist_function("dds_set_qos")
        .whitelist_function("dds_get_listener")
        .whitelist_function("dds_set_listener")
        .whitelist_function("dds_read_status")
        .whitelist_function("dds_create_participant")
        .whitelist_function("dds_create_domain")
        .whitelist_function("dds_get_parent")
        .whitelist_function("dds_get_participant")
        .whitelist_function("dds_get_children")
        .whitelist_function("dds_get_domainid")
        .whitelist_function("dds_lookup_participant")
        .whitelist_function("dds_create_topic")
        .whitelist_function("dds_create_topic_arbitrary")
        .whitelist_function("dds_find_topic")
        .whitelist_function("dds_get_name")
        .whitelist_function("dds_get_type_name")
        .whitelist_function("dds_set_topic_filter")
        .whitelist_function("dds_get_topic_filter")
        .whitelist_function("dds_create_subscriber")
        .whitelist_function("dds_create_publisher")
        .whitelist_function("dds_suspend")
        .whitelist_function("dds_resume")
        .whitelist_function("dds_wait_for_acks")
        .whitelist_function("dds_create_reader")
        .whitelist_function("dds_create_reader_rhc")
        .whitelist_function("dds_reader_wait_for_historical_data")
        .whitelist_function("dds_create_writer")
        .whitelist_function("dds_register_instance")
        .whitelist_function("dds_unregister_instance")
        .whitelist_function("dds_unregister_instance_ih")
        .whitelist_function("dds_unregister_instance_ts")
        .whitelist_function("dds_unregister_instance_ih_ts")
        .whitelist_function("dds_writedispose")
        .whitelist_function("dds_writedispose_ts")
        .whitelist_function("dds_dispose")
        .whitelist_function("dds_dispose_ts")
        .whitelist_function("dds_dispose_ih")
        .whitelist_function("dds_dispose_ih_ts")
        .whitelist_function("dds_write")
        .whitelist_function("dds_write_flush")
        .whitelist_function("dds_writecdr")
        .whitelist_function("dds_write_ts")
        .whitelist_function("dds_create_readcondition")
        .whitelist_function("dds_create_querycondition")
        .whitelist_function("dds_create_guardcondition")
        .whitelist_function("dds_set_guardcondition")
        .whitelist_function("dds_read_guardcondition")
        .whitelist_function("dds_take_guardcondition")
        .whitelist_function("dds_create_waitset")
        .whitelist_function("dds_waitset_get_entities")
        .whitelist_function("dds_waitset_attach")
        .whitelist_function("dds_waitset_detach")
        .whitelist_function("dds_waitset_set_trigger")
        .whitelist_function("dds_waitset_wait")
        .whitelist_function("dds_waitset_wait_until")
        .whitelist_function("dds_read")
        .whitelist_function("dds_read_wl")
        .whitelist_function("dds_read_mask")
        .whitelist_function("dds_read_mask_wl")
        .whitelist_function("dds_read_instance")
        .whitelist_function("dds_read_instance_wl")
        .whitelist_function("dds_read_mask_wl")
        .whitelist_function("dds_read_instance_mask")
        .whitelist_function("dds_read_instance_mask_wl")
        .whitelist_function("dds_take")
        .whitelist_function("dds_take_wl")
        .whitelist_function("dds_take_mask")
        .whitelist_function("dds_take_mask_wl")
        .whitelist_function("dds_take_cdr")
        .whitelist_function("dds_take_instance")
        .whitelist_function("dds_take_instance_wl")
        .whitelist_function("dds_take_instance_mask")
        .whitelist_function("dds_take_instance_mask_wl")
        .whitelist_function("dds_take_next")
        .whitelist_function("dds_take_next_wl")
        .whitelist_function("dds_read_next")
        .whitelist_function("dds_read_next_wl")
        .whitelist_function("dds_return_loan")
        .whitelist_function("dds_lookup_instance")
        .whitelist_function("dds_instance_get_key")
        .whitelist_function("dds_begin_coherent")
        .whitelist_function("dds_end_coherent")
        .whitelist_function("dds_notify_readers")
        .whitelist_function("dds_triggered")
        .whitelist_function("dds_get_topic")
        .whitelist_function("dds_get_matched_subscriptions")
        .whitelist_function("dds_get_matched_subscription_data")
        .whitelist_function("dds_get_matched_publications")
        .whitelist_function("dds_get_matched_publication_data")
        .whitelist_function("dds_assert_liveliness")   /* DDS Public Listener API Follows */
        .whitelist_function("dds_create_listener")
        .whitelist_function("dds_delete_listener")
        .whitelist_function("dds_reset_listener")
        .whitelist_function("dds_copy_listener")
        .whitelist_function("dds_merge_listener")
        .whitelist_function("dds_lset_inconsistent_topic")
        .whitelist_function("dds_lset_liveliness_lost")
        .whitelist_function("dds_lset_offered_deadline_missed")
        .whitelist_function("dds_lset_offered_incompatible_qos")
        .whitelist_function("dds_lset_data_on_readers")
        .whitelist_function("dds_lset_sample_lost")
        .whitelist_function("dds_lset_data_available")
        .whitelist_function("dds_lset_sample_rejected")
        .whitelist_function("dds_lset_liveliness_changed")
        .whitelist_function("dds_lset_requested_deadline_missed")
        .whitelist_function("dds_lset_requested_incompatible_qos")
        .whitelist_function("dds_lset_offered_incompatible_qos")
        .whitelist_function("dds_lset_publication_matched")
        .whitelist_function("dds_lset_subscription_matched")
        .whitelist_function("dds_lget_inconsistent_topic")
        .whitelist_function("dds_lget_liveliness_lost")
        .whitelist_function("dds_lget_offered_deadline_missed")
        .whitelist_function("dds_lget_offered_incompatible_qos")
        .whitelist_function("dds_lget_data_on_readers")
        .whitelist_function("dds_lget_sample_lost")
        .whitelist_function("dds_lget_data_available")
        .whitelist_function("dds_lget_sample_rejected")
        .whitelist_function("dds_lget_liveliness_changed")
        .whitelist_function("dds_lget_requested_deadline_missed")
        .whitelist_function("dds_lget_requested_incompatible_qos")
        .whitelist_function("dds_lget_publication_matched")
        .whitelist_function("dds_lget_subscription_matched")  /* DDS Public Alloc APIs follow */
        .whitelist_function("dds_alloc")
        .whitelist_function("dds_realloc")
        .whitelist_function("dds_realloc_zero")
        .whitelist_function("dds_free")
        .whitelist_function("dds_string_alloc")
        .whitelist_function("dds_string_dup")
        .whitelist_function("dds_string_free")
        .whitelist_function("dds_sample_free")   /* DDS Public Status APIs follow */
        .whitelist_function("dds_get_inconsistent_topic_status")
        .whitelist_function("dds_get_publication_matched_status")
        .whitelist_function("dds_get_liveliness_lost_status")
        .whitelist_function("dds_get_offered_deadline_missed_status")
        .whitelist_function("dds_get_inconsistent_topic_status")
        .whitelist_function("dds_get_offered_incompatible_qos_status")
        .whitelist_function("dds_get_subscription_matched_status")
        .whitelist_function("dds_get_liveliness_changed_status")
        .whitelist_function("dds_get_sample_rejected_status")
        .whitelist_function("dds_get_sample_lost_status")
        .whitelist_function("dds_get_requested_deadline_missed_status")
        .whitelist_function("dds_get_requested_incompatible_qos_status")
        .whitelist_function("dds_get_inconsistent_topic_status")  /* DDS Public QOS APIs follow */
        .whitelist_function("dds_create_qos")
        .whitelist_function("dds_delete_qos")
        .whitelist_function("dds_reset_qos")
        .whitelist_function("dds_copy_qos")
        .whitelist_function("dds_merge_qos")
        .whitelist_function("dds_qos_equal")
        .whitelist_function("dds_qset_userdata")
        .whitelist_function("dds_qset_topicdata")
        .whitelist_function("dds_qset_groupdata")
        .whitelist_function("dds_qset_durability")
        .whitelist_function("dds_qset_history")
        .whitelist_function("dds_qset_resource_limits")
        .whitelist_function("dds_qset_presentation")
        .whitelist_function("dds_qset_lifespan")
        .whitelist_function("dds_qset_deadline")
        .whitelist_function("dds_qset_latency_budget")
        .whitelist_function("dds_qset_ownership")
        .whitelist_function("dds_qset_ownership_strength")
        .whitelist_function("dds_qset_liveliness")
        .whitelist_function("dds_qset_time_based_filter")
        .whitelist_function("dds_qset_partition")
        .whitelist_function("dds_qset_partition1")
        .whitelist_function("dds_qset_reliability")
        .whitelist_function("dds_qset_transport_priority")
        .whitelist_function("dds_qset_destination_order")
        .whitelist_function("dds_qset_writer_data_lifecycle")
        .whitelist_function("dds_qset_reader_data_lifecycle")
        .whitelist_function("dds_qset_durability_service")
        .whitelist_function("dds_qset_ignorelocal")
        .whitelist_function("dds_qget_userdata")
        .whitelist_function("dds_qget_topicdata")
        .whitelist_function("dds_qget_groupdata")
        .whitelist_function("dds_qget_durability")
        .whitelist_function("dds_qget_history")
        .whitelist_function("dds_qget_resource_limits")
        .whitelist_function("dds_qget_presentation")
        .whitelist_function("dds_qget_lifespan")
        .whitelist_function("dds_qget_deadline")
        .whitelist_function("dds_qget_latency_budget")
        .whitelist_function("dds_qget_ownership")
        .whitelist_function("dds_qget_ownership_strength")
        .whitelist_function("dds_qget_liveliness")
        .whitelist_function("dds_qget_time_based_filter")
        .whitelist_function("dds_qget_partition")
        .whitelist_function("dds_qget_reliability")
        .whitelist_function("dds_qget_transport_priority")
        .whitelist_function("dds_qget_destination_order")
        .whitelist_function("dds_qget_writer_data_lifecycle")
        .whitelist_function("dds_qget_reader_data_lifecyele")
        .whitelist_function("dds_qget_durability_service")
        .whitelist_function("dds_qget_history")
        .whitelist_function("dds_qget_ignorelocal")
        .whitelist_function("dds_qget_history")
        .whitelist_var("DDS_DOMAIN_DEFAULT")


    }

    pub fn generate(include_paths: std::vec::Vec<String>, maybe_sysroot: Option<String>) {
        let mut bindings = bindgen::Builder::default().header("wrapper.h");

        for path in include_paths {
            bindings = bindings.clang_arg(format!("-I{}", path));
        }

        if let Some(sysroot) = maybe_sysroot {
            bindings = bindings.clang_arg(format!("--sysroot={}", sysroot));
        }



        let gen = add_whitelist(bindings).generate().expect("Unable to generate bindings");

        if let Ok(path) = env::var("OUT_DIR") {
          let out_path = PathBuf::from(path);
                  gen
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings");
        } else {
          println!("OUT_DIR not set, not generating bindings");
        }

    }

    pub fn main() {
        for (key, value) in env::vars() {
            println!("{}: {}", key, value);
        }

        let headerloc = find_cyclonedds().unwrap();

        match headerloc {
            HeaderLocation::FromCMakeEnvironment(paths, sysroot) => generate(paths, Some(sysroot)),
            HeaderLocation::FromEnvironment(paths) => generate(paths, None),
        }


    }

}

