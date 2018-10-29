// package: haberdasher
// file: haberdasher.proto

/* tslint:disable */

import * as grpc from "grpc";
import * as haberdasher_pb from "./haberdasher_pb";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";

interface IAgentSubscriberService extends grpc.ServiceDefinition<grpc.UntypedServiceImplementation> {
    establishClient: IAgentSubscriberService_IEstablishClient;
    handleAgentRequests: IAgentSubscriberService_IHandleAgentRequests;
    publishVenueUpdates: IAgentSubscriberService_IPublishVenueUpdates;
}

interface IAgentSubscriberService_IEstablishClient extends grpc.MethodDefinition<haberdasher_pb.EstablishClientRequest, haberdasher_pb.Empty> {
    path: string; // "/haberdasher.AgentSubscriber/EstablishClient"
    requestStream: boolean; // false
    responseStream: boolean; // false
    requestSerialize: grpc.serialize<haberdasher_pb.EstablishClientRequest>;
    requestDeserialize: grpc.deserialize<haberdasher_pb.EstablishClientRequest>;
    responseSerialize: grpc.serialize<haberdasher_pb.Empty>;
    responseDeserialize: grpc.deserialize<haberdasher_pb.Empty>;
}
interface IAgentSubscriberService_IHandleAgentRequests extends grpc.MethodDefinition<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest> {
    path: string; // "/haberdasher.AgentSubscriber/HandleAgentRequests"
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
    establishClient: grpc.handleUnaryCall<haberdasher_pb.EstablishClientRequest, haberdasher_pb.Empty>;
    handleAgentRequests: grpc.handleBidiStreamingCall<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    publishVenueUpdates: grpc.handleClientStreamingCall<haberdasher_pb.Venue, haberdasher_pb.Empty>;
}

export interface IAgentSubscriberClient {
    establishClient(request: haberdasher_pb.EstablishClientRequest, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientUnaryCall;
    establishClient(request: haberdasher_pb.EstablishClientRequest, metadata: grpc.Metadata, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientUnaryCall;
    establishClient(request: haberdasher_pb.EstablishClientRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientUnaryCall;
    handleAgentRequests(): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    handleAgentRequests(options: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    handleAgentRequests(metadata: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    publishVenueUpdates(callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    publishVenueUpdates(metadata: grpc.Metadata, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    publishVenueUpdates(options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    publishVenueUpdates(metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
}

export class AgentSubscriberClient extends grpc.Client implements IAgentSubscriberClient {
    constructor(address: string, credentials: grpc.ChannelCredentials, options?: object);
    public establishClient(request: haberdasher_pb.EstablishClientRequest, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientUnaryCall;
    public establishClient(request: haberdasher_pb.EstablishClientRequest, metadata: grpc.Metadata, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientUnaryCall;
    public establishClient(request: haberdasher_pb.EstablishClientRequest, metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientUnaryCall;
    public handleAgentRequests(options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    public handleAgentRequests(metadata?: grpc.Metadata, options?: Partial<grpc.CallOptions>): grpc.ClientDuplexStream<haberdasher_pb.AgentResponse, haberdasher_pb.AgentRequest>;
    public publishVenueUpdates(callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    public publishVenueUpdates(metadata: grpc.Metadata, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    public publishVenueUpdates(options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
    public publishVenueUpdates(metadata: grpc.Metadata, options: Partial<grpc.CallOptions>, callback: (error: Error | null, response: haberdasher_pb.Empty) => void): grpc.ClientWritableStream<haberdasher_pb.Venue>;
}
