// package: haberdasher
// file: haberdasher.proto

/* tslint:disable */

import * as grpc from "grpc";
import * as haberdasher_pb from "./haberdasher_pb";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";

interface IAgentSubscriberService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
    establish: IAgentSubscriberService_IEstablish;
    publishVenueUpdates: IAgentSubscriberService_IPublishVenueUpdates;
}

interface IAgentSubscriberService_IEstablish extends grpc.MethodDefinition<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest> {
    path: string; // "/haberdasher.AgentSubscriber/Establish"
    requestStream: boolean; // true
    responseStream: boolean; // true
    requestSerialize: grpc.serialize<haberdasher_pb.AgentResponse>;
    requestDeserialize: grpc.deserialize<haberdasher_pb.AgentResponse>;
    responseSerialize: grpc.serialize<haberdasher_pb.AgentRequest>;
    responseDeserialize: grpc.deserialize<haberdasher_pb.AgentRequest>;
}
interface IAgentSubscriberService_IPublishVenueUpdates extends grpc.MethodDefinition<haberdasher_pb.Venue, haberdasher_pb.Empty> {
    path: string; // "/haberdasher.AgentSubscriber/PublishVenueUpdates"
    requestStream: boolean; // true
    responseStream: boolean; // false
    requestSerialize: grpc.serialize<haberdasher_pb.Venue>;
    requestDeserialize: grpc.deserialize<haberdasher_pb.Venue>;
    responseSerialize: grpc.serialize<haberdasher_pb.Empty>;
    responseDeserialize: grpc.deserialize<haberdasher_pb.Empty>;
}

export const AgentSubscriberService: IAgentSubscriberService;

export interface IAgentSubscriberServer {
    establish: grpc.handleBidiStreamingCall<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    publishVenueUpdates: grpc.handleClientStreamingCall<haberdasher_pb.Venue, haberdasher_pb.Empty>;
}

export interface IAgentSubscriberClient {
    establish(): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    establish(options: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    establish(metadata: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    publishVenueUpdates(callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    publishVenueUpdates(metadata: grpc.Metadata, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    publishVenueUpdates(options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    publishVenueUpdates(metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
}

export class AgentSubscriberClient extends grpc.Client implements IAgentSubscriberClient {
    constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
    public establish(options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    public establish(metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    public publishVenueUpdates(callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    public publishVenueUpdates(metadata: grpc.Metadata, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    public publishVenueUpdates(options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    public publishVenueUpdates(metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
}
