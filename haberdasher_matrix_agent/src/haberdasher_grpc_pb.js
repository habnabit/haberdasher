// GENERATED CODE -- DO NOT EDIT!

'use strict';
var grpc = require('grpc');
var haberdasher_pb = require('./haberdasher_pb.js');
var google_protobuf_timestamp_pb = require('google-protobuf/google/protobuf/timestamp_pb.js');

function serialize_haberdasher_AgentRequest(arg) {
  if (!(arg instanceof haberdasher_pb.AgentRequest)) {
    throw new Error('Expected argument of type haberdasher.AgentRequest');
  }
  return new Buffer(arg.serializeBinary());
}

function deserialize_haberdasher_AgentRequest(buffer_arg) {
  return haberdasher_pb.AgentRequest.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_haberdasher_AgentResponse(arg) {
  if (!(arg instanceof haberdasher_pb.AgentResponse)) {
    throw new Error('Expected argument of type haberdasher.AgentResponse');
  }
  return new Buffer(arg.serializeBinary());
}

function deserialize_haberdasher_AgentResponse(buffer_arg) {
  return haberdasher_pb.AgentResponse.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_haberdasher_Empty(arg) {
  if (!(arg instanceof haberdasher_pb.Empty)) {
    throw new Error('Expected argument of type haberdasher.Empty');
  }
  return new Buffer(arg.serializeBinary());
}

function deserialize_haberdasher_Empty(buffer_arg) {
  return haberdasher_pb.Empty.deserializeBinary(new Uint8Array(buffer_arg));
}

function serialize_haberdasher_Venue(arg) {
  if (!(arg instanceof haberdasher_pb.Venue)) {
    throw new Error('Expected argument of type haberdasher.Venue');
  }
  return new Buffer(arg.serializeBinary());
}

function deserialize_haberdasher_Venue(buffer_arg) {
  return haberdasher_pb.Venue.deserializeBinary(new Uint8Array(buffer_arg));
}


var AgentSubscriberService = exports.AgentSubscriberService = {
  handleAgentRequests: {
    path: '/haberdasher.AgentSubscriber/HandleAgentRequests',
    requestStream: true,
    responseStream: true,
    requestType: haberdasher_pb.AgentResponse,
    responseType: haberdasher_pb.AgentRequest,
    requestSerialize: serialize_haberdasher_AgentResponse,
    requestDeserialize: deserialize_haberdasher_AgentResponse,
    responseSerialize: serialize_haberdasher_AgentRequest,
    responseDeserialize: deserialize_haberdasher_AgentRequest,
  },
  publishVenueUpdates: {
    path: '/haberdasher.AgentSubscriber/PublishVenueUpdates',
    requestStream: true,
    responseStream: false,
    requestType: haberdasher_pb.Venue,
    responseType: haberdasher_pb.Empty,
    requestSerialize: serialize_haberdasher_Venue,
    requestDeserialize: deserialize_haberdasher_Venue,
    responseSerialize: serialize_haberdasher_Empty,
    responseDeserialize: deserialize_haberdasher_Empty,
  },
};

exports.AgentSubscriberClient = grpc.makeGenericClientConstructor(AgentSubscriberService);
var AgentAggregatorService = exports.AgentAggregatorService = {
  subscribeToVenueUpdates: {
    path: '/haberdasher.AgentAggregator/SubscribeToVenueUpdates',
    requestStream: false,
    responseStream: true,
    requestType: haberdasher_pb.Empty,
    responseType: haberdasher_pb.Venue,
    requestSerialize: serialize_haberdasher_Empty,
    requestDeserialize: deserialize_haberdasher_Empty,
    responseSerialize: serialize_haberdasher_Venue,
    responseDeserialize: deserialize_haberdasher_Venue,
  },
};

exports.AgentAggregatorClient = grpc.makeGenericClientConstructor(AgentAggregatorService);
