---types---

manual.rpc_result#f35c6d01 req_msg_id:long result:TypedObject = manual.RpcResult;
manual.basic_message msg_id:long seqno:int body:lengthPrefixedTypedObject = manual.BasicMessage;
manual.msg_container#73f1f8dc messages:vector<manual.basic_message> = manual.MessageContainer;
manual.gzip_packed#3072cfa1 packed_data:bytes = manual.GzipPacked;

manual.bind_auth_key_inner#75a3f765 nonce:long temp_auth_key_id:long perm_auth_key_id:long temp_session_id:long expires_at:int = manual.BindAuthKeyInner;

wire.outbound_encrypted salt:long session_id:long message_id:long seq_no:int payload:lengthPrefixedTypedObject = wire.OutboundEncrypted;
wire.inbound_encrypted salt:long session_id:long message_id:long seq_no:int payload_len:int = wire.InboundEncrypted;
wire.outbound_raw auth_key_id:long message_id:long payload:lengthPrefixedTypedObject = wire.OutboundRaw;
wire.inbound_raw auth_key_id:long message_id:long payload_len:int = wire.InboundRaw;
