# BIFs

- [x] ubif erlang:abs/1
- [ ] bif erlang:adler32/1
- [ ] bif erlang:adler32/2
- [ ] bif erlang:adler32_combine/3
- [ ] bif erlang:apply/3
- [ ] bif erlang:atom_to_list/1
- [ ] bif erlang:binary_to_list/1
- [ ] bif erlang:binary_to_list/3
- [ ] bif erlang:binary_to_term/1
- [ ] bif erlang:crc32/1
- [ ] bif erlang:crc32/2
- [ ] bif erlang:crc32_combine/3
- [x] bif erlang:date/0
- [ ] bif erlang:delete_module/1
- [ ] bif erlang:display/1
- [ ] bif erlang:display_string/1
- [ ] bif erlang:display_nl/0
- [ ] ubif erlang:element/2
- [x] bif erlang:erase/0
- [x] bif erlang:erase/1
- [ ] bif erlang:exit/1
- [ ] bif erlang:exit/2
- [ ] bif erlang:exit_signal/2
- [ ] bif erlang:external_size/1
- [ ] bif erlang:external_size/2
- [ ] ubif erlang:float/1
- [ ] bif erlang:float_to_list/1
- [ ] bif erlang:float_to_list/2
- [ ] bif erlang:fun_info/2
- [ ] bif erts_internal:garbage_collect/1
- [x] bif erlang:get/0
- [x] bif erlang:get/1
- [x] bif erlang:get_keys/1
- [ ] bif erlang:group_leader/0
- [ ] bif erts_internal:group_leader/2
- [ ] bif erts_internal:group_leader/3
- [ ] bif erlang:halt/2
- [ ] bif erlang:phash/2
- [ ] bif erlang:phash2/1
- [ ] bif erlang:phash2/2
- [x] ubif erlang:hd/1
- [ ] bif erlang:integer_to_list/1
- [ ] bif erlang:is_alive/0
- [ ] ubif erlang:length/1
- [ ] bif erlang:link/1
- [ ] bif erlang:list_to_atom/1
- [ ] bif erlang:list_to_binary/1
- [ ] bif erlang:list_to_float/1
- [ ] bif erlang:list_to_integer/1
- [ ] bif erlang:list_to_pid/1
- [ ] bif erlang:list_to_port/1
- [ ] bif erlang:list_to_ref/1
- [ ] bif erlang:list_to_tuple/1
- [ ] bif erlang:loaded/0
- [x] bif erlang:localtime/0
- [ ] bif erlang:localtime_to_universaltime/2
- [ ] bif erlang:make_ref/0
- [ ] bif erlang:unique_integer/0
- [ ] bif erlang:unique_integer/1
- [ ] bif erlang:md5/1
- [ ] bif erlang:md5_init/0
- [ ] bif erlang:md5_update/2
- [ ] bif erlang:md5_final/1
- [ ] bif erlang:module_loaded/1
- [ ] bif erlang:function_exported/3
- [ ] bif erlang:monitor_node/2
- [ ] bif erlang:monitor_node/3
- [ ] ubif erlang:node/1
- [ ] ubif erlang:node/0
- [ ] bif erlang:nodes/1
- [ ] bif erlang:now/0
- [x] bif erlang:monotonic_time/0
- [ ] bif erlang:monotonic_time/1
- [x] bif erlang:system_time/0
- [ ] bif erlang:system_time/1
- [ ] bif erlang:time_offset/0
- [ ] bif erlang:time_offset/1
- [ ] bif erlang:timestamp/0
- [ ] bif erts_internal:open_port/2
- [ ] bif erlang:pid_to_list/1
- [ ] bif erlang:ports/0
- [ ] bif erlang:pre_loaded/0
- [ ] bif erlang:process_flag/2
- [ ] bif erts_internal:process_flag/3
- [ ] bif erlang:process_info/1
- [ ] bif erlang:process_info/2
- [ ] bif erlang:processes/0
- [x] bif erlang:put/2
- [ ] bif erlang:register/2
- [ ] bif erlang:registered/0
- [ ] ubif erlang:round/1
- [x] ubif erlang:self/0
- [ ] bif erlang:setelement/3
- [ ] ubif erlang:size/1
- [x] bif erlang:spawn/3
- [ ] bif erlang:spawn_link/3
- [ ] bif erlang:split_binary/2
- [ ] bif erlang:statistics/1
- [ ] bif erlang:term_to_binary/1
- [ ] bif erlang:term_to_binary/2
- [ ] bif erlang:throw/1
- [ ] bif erlang:time/0
- [x] ubif erlang:tl/1
- [x] ubif erlang:trunc/1
- [ ] bif erlang:tuple_to_list/1
- [x] bif erlang:universaltime/0
- [ ] bif erlang:universaltime_to_localtime/1
- [ ] bif erlang:unlink/1
- [ ] bif erlang:unregister/1
- [ ] bif erlang:whereis/1
- [ ] bif erlang:spawn_opt/1
- [ ] bif erlang:setnode/2
- [ ] bif erlang:dist_get_stat/1
- [ ] bif erlang:dist_ctrl_input_handler/2
- [ ] bif erlang:dist_ctrl_put_data/2
- [ ] bif erlang:dist_ctrl_get_data/1
- [ ] bif erlang:dist_ctrl_get_data_notification/1

#### Static native functions in erts_internal
- [ ] bif erts_internal:port_info/1
- [ ] bif erts_internal:port_info/2
- [ ] bif erts_internal:port_call/3
- [ ] bif erts_internal:port_command/3
- [ ] bif erts_internal:port_control/3
- [ ] bif erts_internal:port_close/1
- [ ] bif erts_internal:port_connect/2
- [ ] bif erts_internal:request_system_task/3
- [ ] bif erts_internal:request_system_task/4
- [ ] bif erts_internal:check_process_code/1
- [ ] bif erts_internal:map_to_tuple_keys/1
- [ ] bif erts_internal:term_type/1
- [ ] bif erts_internal:map_hashmap_children/1
- [ ] bif erts_internal:time_unit/0
- [ ] bif erts_internal:perf_counter_unit/0
- [ ] bif erts_internal:is_system_process/1
- [ ] bif erts_internal:system_check/1
- [ ] bif erts_internal:release_literal_area_switch/0
- [ ] bif erts_internal:scheduler_wall_time/1
- [ ] bif erts_internal:dirty_process_handle_signals/1
- [ ] bif erts_internal:create_dist_channel/4

#### inet_db support
- [ ] bif erlang:port_set_data/2
- [ ] bif erlang:port_get_data/1

#### Tracing & debugging.
- [ ] bif erts_internal:trace_pattern/3
- [ ] bif erts_internal:trace/3
- [ ] bif erlang:trace_info/2
- [ ] bif erlang:trace_delivered/1
- [ ] bif erlang:seq_trace/2
- [ ] bif erlang:seq_trace_info/1
- [ ] bif erlang:seq_trace_print/1
- [ ] bif erlang:seq_trace_print/2
- [ ] bif erts_internal:suspend_process/2
- [ ] bif erlang:resume_process/1
- [ ] bif erts_internal:process_display/2
- [ ] bif erlang:bump_reductions/1

#### Math
- [x] bif math:cos/1
- [x] bif math:cosh/1
- [x] bif math:sin/1
- [x] bif math:sinh/1
- [x] bif math:tan/1
- [x] bif math:tanh/1
- [x] bif math:acos/1
- [x] bif math:acosh/1
- [x] bif math:asin/1
- [x] bif math:asinh/1
- [x] bif math:atan/1
- [x] bif math:atanh/1
- [ ] bif math:erf/1
- [ ] bif math:erfc/1
- [ ] bif math:exp/1
- [x] bif math:log/1
- [x] bif math:log2/1
- [x] bif math:log10/1
- [x] bif math:sqrt/1
- [ ] bif math:atan2/2
- [ ] bif math:pow/2
#### Timers
- [ ] bif erlang:start_timer/3
- [ ] bif erlang:start_timer/4
- [ ] bif erlang:send_after/3
- [ ] bif erlang:send_after/4
- [ ] bif erlang:cancel_timer/1
- [ ] bif erlang:cancel_timer/2
- [ ] bif erlang:read_timer/1
- [ ] bif erlang:read_timer/2
#### Tuples
- [ ] bif erlang:make_tuple/2
- [ ] bif erlang:append_element/2
- [ ] bif erlang:make_tuple/3
#### System
- [ ] bif erlang:system_flag/2
- [ ] bif erlang:system_info/1
#### New in R9C
- [ ] bif erlang:system_monitor/0
- [ ] bif erlang:system_monitor/1
- [ ] bif erlang:system_monitor/2
- [ ] bif erlang:system_profile/2
- [ ] bif erlang:system_profile/0
- [ ] bif erlang:ref_to_list/1
- [ ] bif erlang:port_to_list/1
- [ ] bif erlang:fun_to_list/1

- [ ] bif erlang:monitor/2
- [ ] bif erlang:demonitor/1
- [ ] bif erlang:demonitor/2

- [ ] bif erlang:is_process_alive/1
- [ ] bif erts_internal:is_process_alive/2

- [ ] bif erlang:error/1		error_1
- [ ] bif erlang:error/2		error_2
- [ ] bif erlang:raise/3		raise_3
- [ ] bif erlang:get_stacktrace/0

- [ ] bif erlang:is_builtin/3

- [ ] ubif erlang:'and'/2
- [ ] ubif erlang:'or'/2
- [ ] ubif erlang:'xor'/2
- [ ] ubif erlang:'not'/1

- [ ] ubif erlang:'>'/2			sgt_2
- [ ] ubif erlang:'>='/2			sge_2
- [ ] ubif erlang:'<'/2			slt_2
- [ ] ubif erlang:'=<'/2			sle_2
- [ ] ubif erlang:'=:='/2			seq_2
- [ ] ubif erlang:'=='/2			seqeq_2
- [ ] ubif erlang:'=/='/2			sneq_2
- [ ] ubif erlang:'/='/2			sneqeq_2
- [x] ubif erlang:'+'/2			splus_2
- [x] ubif erlang:'-'/2			sminus_2
- [x] ubif erlang:'*'/2			stimes_2
- [ ] ubif erlang:'/'/2			div_2
- [x] ubif erlang:'div'/2			intdiv_2
- [x] ubif erlang:'rem'/2
- [ ] ubif erlang:'bor'/2
- [ ] ubif erlang:'band'/2
- [ ] ubif erlang:'bxor'/2
- [ ] ubif erlang:'bsl'/2
- [ ] ubif erlang:'bsr'/2
- [ ] ubif erlang:'bnot'/1
- [ ] ubif erlang:'-'/1			sminus_1
- [ ] ubif erlang:'+'/1			splus_1


#### New operators
- [ ] bif erlang:'!'/2		ebif_bang_2
- [x] bif erlang:send/2
- [ ] bif erlang:send/3
- [ ] bif erlang:'++'/2		ebif_plusplus_2
- [ ] bif erlang:append/2
- [ ] bif erlang:'--'/2		ebif_minusminus_2
- [ ] bif erlang:subtract/2

- [x] ubif erlang:is_atom/1
- [x] ubif erlang:is_list/1
- [x] ubif erlang:is_tuple/1
- [x] ubif erlang:is_float/1
- [x] ubif erlang:is_integer/1
- [x] ubif erlang:is_number/1
- [x] ubif erlang:is_pid/1
- [x] ubif erlang:is_port/1
- [x] ubif erlang:is_reference/1
- [x] ubif erlang:is_binary/1
- [x] ubif erlang:is_function/1
- [ ] ubif erlang:is_function/2
- [ ] ubif erlang:is_record/2
- [ ] ubif erlang:is_record/3

- [ ] bif erlang:match_spec_test/3

#### ETS
- [ ] bif ets:internal_request_all/0
- [ ] bif ets:new/2
- [ ] bif ets:delete/1
- [ ] bif ets:delete/2
- [ ] bif ets:delete_object/2
- [ ] bif ets:first/1
- [ ] bif ets:is_compiled_ms/1
- [ ] bif ets:lookup/2
- [ ] bif ets:lookup_element/3
- [ ] bif ets:info/1
- [ ] bif ets:info/2
- [ ] bif ets:last/1
- [ ] bif ets:match/1
- [ ] bif ets:match/2
- [ ] bif ets:match/3
- [ ] bif ets:match_object/1
- [ ] bif ets:match_object/2
- [ ] bif ets:match_object/3
- [ ] bif ets:member/2
- [ ] bif ets:next/2
- [ ] bif ets:prev/2
- [ ] bif ets:insert/2
- [ ] bif ets:insert_new/2
- [ ] bif ets:rename/2
- [ ] bif ets:safe_fixtable/2
- [ ] bif ets:slot/2
- [ ] bif ets:update_counter/3
- [ ] bif ets:select/1
- [ ] bif ets:select/2
- [ ] bif ets:select/3
- [ ] bif ets:select_count/2
- [ ] bif ets:select_reverse/1
- [ ] bif ets:select_reverse/2
- [ ] bif ets:select_reverse/3
- [ ] bif ets:select_replace/2
- [ ] bif ets:match_spec_compile/1
- [ ] bif ets:match_spec_run_r/3

#### OS
- [ ] bif os:get_env_var/1
- [ ] bif os:set_env_var/2
- [ ] bif os:unset_env_var/1
- [ ] bif os:list_env_vars/0
- [ ] bif os:getpid/0
- [ ] bif os:timestamp/0
- [ ] bif os:system_time/0
- [ ] bif os:system_time/1
- [ ] bif os:perf_counter/0

#### Bifs in the erl_ddll module (the module actually does not exist)
- [ ] bif erl_ddll:try_load/3
- [ ] bif erl_ddll:try_unload/2
- [ ] bif erl_ddll:loaded_drivers/0
- [ ] bif erl_ddll:info/2
- [ ] bif erl_ddll:format_error_int/1
- [ ] bif erl_ddll:monitor/2
- [ ] bif erl_ddll:demonitor/1

#### Bifs in the re module 
- [ ] bif re:version/0
- [ ] bif re:compile/1
- [ ] bif re:compile/2
- [ ] bif re:run/2
- [ ] bif re:run/3

#### Bifs in lists module.
- [x] bif lists:member/2
- [ ] bif lists:reverse/2
- [x] bif lists:keymember/3
- [x] bif lists:keysearch/3
- [x] bif lists:keyfind/3

#### Bifs for debugging.
- [ ] bif erts_debug:disassemble/1
- [ ] bif erts_debug:breakpoint/2
- [ ] bif erts_debug:same/2
- [ ] bif erts_debug:flat_size/1
- [ ] bif erts_debug:get_internal_state/1
- [ ] bif erts_debug:set_internal_state/2
- [ ] bif erts_debug:display/1
- [ ] bif erts_debug:dist_ext_to_term/2
- [ ] bif erts_debug:instructions/0
- [ ] bif erts_debug:dirty_cpu/2
- [ ] bif erts_debug:dirty_io/2
- [ ] bif erts_debug:dirty/3

#### Lock counter bif's
- [ ] bif erts_debug:lcnt_control/2
- [ ] bif erts_debug:lcnt_control/1
- [ ] bif erts_debug:lcnt_collect/0
- [ ] bif erts_debug:lcnt_clear/0

#### New Bifs in R8.
- [ ] bif code:get_chunk/2
- [ ] bif code:module_md5/1
- [ ] bif code:make_stub_module/3
- [ ] bif code:is_module_native/1
#### New Bifs in R9C.
- [ ] bif erlang:hibernate/3
- [ ] bif error_logger:warning_map/0

#### New Bifs in R10B.
- [ ] bif erlang:get_module_info/1
- [ ] bif erlang:get_module_info/2
- [x] ubif erlang:is_boolean/1
- [ ] bif string:list_to_integer/1
- [ ] bif string:list_to_float/1
- [ ] bif erlang:make_fun/3
- [ ] bif erlang:iolist_size/1
- [ ] bif erlang:iolist_to_binary/1
- [ ] bif erlang:list_to_existing_atom/1

#### New Bifs in R12B-0
- [ ] ubif erlang:is_bitstring/1
- [ ] ubif erlang:tuple_size/1
- [ ] ubif erlang:byte_size/1
- [ ] ubif erlang:bit_size/1
- [ ] bif erlang:list_to_bitstring/1
- [ ] bif erlang:bitstring_to_list/1

#### New Bifs in R12B-2
- [ ] bif ets:update_element/3

#### New Bifs in R12B-4
- [ ] bif erlang:decode_packet/3

#### New Bifs in R12B-5
- [ ] bif unicode:characters_to_binary/2
- [ ] bif unicode:characters_to_list/2
- [ ] bif unicode:bin_is_7bit/1

#### New Bifs in R13A.
- [ ] bif erlang:atom_to_binary/2
- [ ] bif erlang:binary_to_atom/2
- [ ] bif erlang:binary_to_existing_atom/2
- [ ] bif net_kernel:dflag_unicode_io/1

#### New Bifs in R13B-1
- [ ] bif ets:give_away/3
- [ ] bif ets:setopts/2

#### New Bifs in R13B3
- [ ] bif erlang:load_nif/2
- [ ] bif erlang:call_on_load_function/1
- [ ] bif erlang:finish_after_on_load/2

#### New Bifs in R13B04
- [ ] bif erlang:binary_to_term/2

#### The binary match bifs (New in R14A - EEP9)
##### The searching/splitting/substituting thingies
- [ ] ubif erlang:binary_part/2
- [ ] ubif erlang:binary_part/3
- [ ] bif binary:compile_pattern/1
- [ ] bif binary:match/2
- [ ] bif binary:match/3
- [ ] bif binary:matches/2
- [ ] bif binary:matches/3
- [ ] bif binary:longest_common_prefix/1
- [ ] bif binary:longest_common_suffix/1
- [ ] bif binary:first/1
- [ ] bif binary:last/1
- [ ] bif binary:at/2
- [ ] bif binary:part/2 binary_binary_part_2
- [ ] bif binary:part/3 binary_binary_part_3
- [ ] bif binary:list_to_bin/1
- [ ] bif binary:copy/1
- [ ] bif binary:copy/2
- [ ] bif binary:referenced_byte_size/1
- [ ] bif binary:encode_unsigned/1
- [ ] bif binary:encode_unsigned/2
- [ ] bif binary:decode_unsigned/1
- [ ] bif binary:decode_unsigned/2
- [ ] bif erlang:nif_error/1
- [ ] bif erlang:nif_error/2

#### Helpers for unicode filenames
- [ ] bif prim_file:internal_name2native/1
- [ ] bif prim_file:internal_native2name/1
- [ ] bif prim_file:internal_normalize_utf8/1
- [ ] bif prim_file:is_translatable/1
- [ ] bif file:native_name_encoding/0

#### New in R14B04.
- [ ] bif erlang:check_old_code/1

#### New in R15B
- [ ] bif erlang:universaltime_to_posixtime/1
- [ ] bif erlang:posixtime_to_universaltime/1

#### New in R15B01
# The dtrace BIF's are always present, but give dummy results if dynamic trace is not enabled in the build
- [ ] bif erlang:dt_put_tag/1
- [ ] bif erlang:dt_get_tag/0
- [ ] bif erlang:dt_get_tag_data/0
- [ ] bif erlang:dt_spread_tag/1
- [ ] bif erlang:dt_restore_tag/1
##### These are dummies even with enabled dynamic trace unless vm probes are enabled. 
##### They are also internal, for dtrace tags sent to the VM's own drivers (efile)
- [ ] bif erlang:dt_prepend_vm_tag_data/1
- [ ] bif erlang:dt_append_vm_tag_data/1

#### New in R16B.
- [ ] bif erlang:prepare_loading/2
- [ ] bif erlang:finish_loading/1
- [ ] bif erlang:insert_element/3
- [ ] bif erlang:delete_element/2
- [ ] bif erlang:binary_to_integer/1
- [ ] bif erlang:binary_to_integer/2
- [ ] bif erlang:integer_to_binary/1
- [ ] bif erlang:list_to_integer/2
- [ ] bif erlang:float_to_binary/1
- [ ] bif erlang:float_to_binary/2
- [ ] bif erlang:binary_to_float/1
- [ ] bif io:printable_range/0

#### New in 17.0
- [ ] bif re:inspect/2
- [ ] ubif erlang:is_map/1
- [ ] ubif erlang:map_size/1
- [ ] bif maps:find/2
- [ ] bif maps:get/2
- [ ] bif maps:from_list/1
- [ ] bif maps:is_key/2
- [ ] bif maps:keys/1
- [ ] bif maps:merge/2
- [ ] bif maps:put/3
- [ ] bif maps:remove/2
- [ ] bif maps:update/3
- [ ] bif maps:values/1
- [ ] bif erts_internal:cmp_term/2
- [ ] bif ets:take/2

#### New in 17.1
- [ ] bif erlang:fun_info_mfa/1

#### New in 18.0
- [x] bif erlang:get_keys/0
- [ ] bif ets:update_counter/4
- [ ] bif erts_debug:map_info/1

#### New in 19.0
- [ ] bif erts_internal:is_process_executing_dirty/1
- [ ] bif erts_internal:check_dirty_process_code/2
- [ ] bif erts_internal:purge_module/2
- [ ] bif binary:split/2
- [ ] bif binary:split/3
- [ ] bif erts_debug:size_shared/1
- [ ] bif erts_debug:copy_shared/1
- [ ] bif erlang:has_prepared_code_on_load/1
- [ ] bif maps:take/2

### New in 20.0
- [ ] ubif erlang:floor/1
- [ ] ubif erlang:ceil/1
- [ ] bif math:floor/1
- [ ] bif math:ceil/1
- [ ] bif math:fmod/2
- [ ] bif os:set_signal/2

### New in 20.1
- [ ] bif erlang:iolist_to_iovec/1

### New in 21.0
- [ ] bif erts_internal:get_dflags/0
- [ ] bif erts_internal:new_connection/1
- [ ] bif erts_internal:abort_connection/2
- [ ] bif erts_internal:map_next/3
- [ ] bif ets:whereis/1
- [ ] bif erts_internal:gather_alloc_histograms/1
- [ ] bif erts_internal:gather_carrier_info/1
- [ ] ubif erlang:map_get/2
- [ ] ubif erlang:is_map_key/2
- [ ] bif ets:internal_delete_all/2
- [ ] bif ets:internal_select_delete/2

#### New in 21.2
##### Persistent terms
- [ ] bif persistent_term:put/2
- [ ] bif persistent_term:get/1
- [ ] bif persistent_term:get/0
- [ ] bif persistent_term:erase/1
- [ ] bif persistent_term:info/0
- [ ] bif erts_internal:erase_persistent_terms/0
##### Atomics
- [ ] bif erts_internal:atomics_new/2
- [ ] bif atomics:get/2
- [ ] bif atomics:put/3
- [ ] bif atomics:add/3
- [ ] bif atomics:add_get/3
- [ ] bif atomics:exchange/3
- [ ] bif atomics:compare_exchange/4
- [ ] bif atomics:info/1
##### Counters
- [ ] bif erts_internal:counters_new/1
- [ ] bif erts_internal:counters_get/2
- [ ] bif erts_internal:counters_add/3
- [ ] bif erts_internal:counters_put/3
- [ ] bif erts_internal:counters_info/1
