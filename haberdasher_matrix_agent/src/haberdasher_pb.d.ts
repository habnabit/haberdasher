// package: haberdasher
// file: haberdasher.proto

/* tslint:disable */

import * as jspb from "google-protobuf";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";

export class Empty extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Empty.AsObject;
    static toObject(includeInstance: boolean, msg: Empty): Empty.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Empty, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Empty;
    static deserializeBinaryFromReader(message: Empty, reader: jspb.BinaryReader): Empty;
}

export namespace Empty {
    export type AsObject = {
    }
}

export class AgentRequest extends jspb.Message { 
    getSeqno(): number;
    setSeqno(value: number): void;


    hasListVenues(): boolean;
    clearListVenues(): void;
    getListVenues(): ListVenuesRequest | undefined;
    setListVenues(value?: ListVenuesRequest): void;


    getKindCase(): AgentRequest.KindCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AgentRequest.AsObject;
    static toObject(includeInstance: boolean, msg: AgentRequest): AgentRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AgentRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AgentRequest;
    static deserializeBinaryFromReader(message: AgentRequest, reader: jspb.BinaryReader): AgentRequest;
}

export namespace AgentRequest {
    export type AsObject = {
        seqno: number,
        listVenues?: ListVenuesRequest.AsObject,
    }

    export enum KindCase {
        KIND_NOT_SET = 0,
    
    LIST_VENUES = 2,

    }

}

export class AgentResponse extends jspb.Message { 
    getSeqno(): number;
    setSeqno(value: number): void;


    hasError(): boolean;
    clearError(): void;
    getError(): AgentErrorResponse | undefined;
    setError(value?: AgentErrorResponse): void;


    hasListVenues(): boolean;
    clearListVenues(): void;
    getListVenues(): ListVenuesResponse | undefined;
    setListVenues(value?: ListVenuesResponse): void;


    getKindCase(): AgentResponse.KindCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AgentResponse.AsObject;
    static toObject(includeInstance: boolean, msg: AgentResponse): AgentResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AgentResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AgentResponse;
    static deserializeBinaryFromReader(message: AgentResponse, reader: jspb.BinaryReader): AgentResponse;
}

export namespace AgentResponse {
    export type AsObject = {
        seqno: number,
        error?: AgentErrorResponse.AsObject,
        listVenues?: ListVenuesResponse.AsObject,
    }

    export enum KindCase {
        KIND_NOT_SET = 0,
    
    ERROR = 2,

    LIST_VENUES = 3,

    }

}

export class AgentErrorResponse extends jspb.Message { 
    getMessage(): string;
    setMessage(value: string): void;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AgentErrorResponse.AsObject;
    static toObject(includeInstance: boolean, msg: AgentErrorResponse): AgentErrorResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AgentErrorResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AgentErrorResponse;
    static deserializeBinaryFromReader(message: AgentErrorResponse, reader: jspb.BinaryReader): AgentErrorResponse;
}

export namespace AgentErrorResponse {
    export type AsObject = {
        message: string,
    }
}

export class ListVenuesRequest extends jspb.Message { 

    hasWithUpdatesSince(): boolean;
    clearWithUpdatesSince(): void;
    getWithUpdatesSince(): google_protobuf_timestamp_pb.Timestamp | undefined;
    setWithUpdatesSince(value?: google_protobuf_timestamp_pb.Timestamp): void;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ListVenuesRequest.AsObject;
    static toObject(includeInstance: boolean, msg: ListVenuesRequest): ListVenuesRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ListVenuesRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ListVenuesRequest;
    static deserializeBinaryFromReader(message: ListVenuesRequest, reader: jspb.BinaryReader): ListVenuesRequest;
}

export namespace ListVenuesRequest {
    export type AsObject = {
        withUpdatesSince?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    }
}

export class ListVenuesResponse extends jspb.Message { 
    clearVenuesList(): void;
    getVenuesList(): Array<Venue>;
    setVenuesList(value: Array<Venue>): void;
    addVenues(value?: Venue, index?: number): Venue;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ListVenuesResponse.AsObject;
    static toObject(includeInstance: boolean, msg: ListVenuesResponse): ListVenuesResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ListVenuesResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ListVenuesResponse;
    static deserializeBinaryFromReader(message: ListVenuesResponse, reader: jspb.BinaryReader): ListVenuesResponse;
}

export namespace ListVenuesResponse {
    export type AsObject = {
        venuesList: Array<Venue.AsObject>,
    }
}

export class Agent extends jspb.Message { 
    getName(): string;
    setName(value: string): void;

    getVersion(): string;
    setVersion(value: string): void;

    getRevision(): string;
    setRevision(value: string): void;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Agent.AsObject;
    static toObject(includeInstance: boolean, msg: Agent): Agent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Agent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Agent;
    static deserializeBinaryFromReader(message: Agent, reader: jspb.BinaryReader): Agent;
}

export namespace Agent {
    export type AsObject = {
        name: string,
        version: string,
        revision: string,
    }
}

export class Individual extends jspb.Message { 
    getId(): string;
    setId(value: string): void;

    getName(): string;
    setName(value: string): void;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Individual.AsObject;
    static toObject(includeInstance: boolean, msg: Individual): Individual.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Individual, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Individual;
    static deserializeBinaryFromReader(message: Individual, reader: jspb.BinaryReader): Individual;
}

export namespace Individual {
    export type AsObject = {
        id: string,
        name: string,
    }
}

export class Group extends jspb.Message { 
    getId(): string;
    setId(value: string): void;

    getName(): string;
    setName(value: string): void;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Group.AsObject;
    static toObject(includeInstance: boolean, msg: Group): Group.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Group, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Group;
    static deserializeBinaryFromReader(message: Group, reader: jspb.BinaryReader): Group;
}

export namespace Group {
    export type AsObject = {
        id: string,
        name: string,
    }
}

export class Performer extends jspb.Message { 

    hasIndividual(): boolean;
    clearIndividual(): void;
    getIndividual(): Individual | undefined;
    setIndividual(value?: Individual): void;


    hasService(): boolean;
    clearService(): void;
    getService(): boolean;
    setService(value: boolean): void;


    hasMyself(): boolean;
    clearMyself(): void;
    getMyself(): boolean;
    setMyself(value: boolean): void;


    getKindCase(): Performer.KindCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Performer.AsObject;
    static toObject(includeInstance: boolean, msg: Performer): Performer.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Performer, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Performer;
    static deserializeBinaryFromReader(message: Performer, reader: jspb.BinaryReader): Performer;
}

export namespace Performer {
    export type AsObject = {
        individual?: Individual.AsObject,
        service: boolean,
        myself: boolean,
    }

    export enum KindCase {
        KIND_NOT_SET = 0,
    
    INDIVIDUAL = 1,

    SERVICE = 2,

    MYSELF = 3,

    }

}

export class Origin extends jspb.Message { 
    clearPathList(): void;
    getPathList(): Array<Origin.Segment>;
    setPathList(value: Array<Origin.Segment>): void;
    addPath(value?: Origin.Segment, index?: number): Origin.Segment;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Origin.AsObject;
    static toObject(includeInstance: boolean, msg: Origin): Origin.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Origin, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Origin;
    static deserializeBinaryFromReader(message: Origin, reader: jspb.BinaryReader): Origin;
}

export namespace Origin {
    export type AsObject = {
        pathList: Array<Origin.Segment.AsObject>,
    }


    export class Segment extends jspb.Message { 

    hasIndividual(): boolean;
    clearIndividual(): void;
    getIndividual(): Individual | undefined;
    setIndividual(value?: Individual): void;


    hasGroup(): boolean;
    clearGroup(): void;
    getGroup(): Group | undefined;
    setGroup(value?: Group): void;

    getService(): boolean;
    setService(value: boolean): void;

    getServiceInstance(): string;
    setServiceInstance(value: string): void;

    getMyself(): boolean;
    setMyself(value: boolean): void;

    getThemself(): boolean;
    setThemself(value: boolean): void;

    getAgent(): string;
    setAgent(value: string): void;


        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Segment.AsObject;
        static toObject(includeInstance: boolean, msg: Segment): Segment.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Segment, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Segment;
        static deserializeBinaryFromReader(message: Segment, reader: jspb.BinaryReader): Segment;
    }

    export namespace Segment {
        export type AsObject = {
        individual?: Individual.AsObject,
        group?: Group.AsObject,
        service: boolean,
        serviceInstance: string,
        myself: boolean,
        themself: boolean,
        agent: string,
        }
    }

}

export class Message extends jspb.Message { 

    hasPerformer(): boolean;
    clearPerformer(): void;
    getPerformer(): Origin.Segment | undefined;
    setPerformer(value?: Origin.Segment): void;


    hasAt(): boolean;
    clearAt(): void;
    getAt(): google_protobuf_timestamp_pb.Timestamp | undefined;
    setAt(value?: google_protobuf_timestamp_pb.Timestamp): void;


    hasText(): boolean;
    clearText(): void;
    getText(): string;
    setText(value: string): void;


    hasPose(): boolean;
    clearPose(): void;
    getPose(): string;
    setPose(value: string): void;


    getContentCase(): Message.ContentCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Message.AsObject;
    static toObject(includeInstance: boolean, msg: Message): Message.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Message, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Message;
    static deserializeBinaryFromReader(message: Message, reader: jspb.BinaryReader): Message;
}

export namespace Message {
    export type AsObject = {
        performer?: Origin.Segment.AsObject,
        at?: google_protobuf_timestamp_pb.Timestamp.AsObject,
        text: string,
        pose: string,
    }

    export enum ContentCase {
        CONTENT_NOT_SET = 0,
    
    TEXT = 3,

    POSE = 4,

    }

}

export class Venue extends jspb.Message { 

    hasOrigin(): boolean;
    clearOrigin(): void;
    getOrigin(): Origin | undefined;
    setOrigin(value?: Origin): void;


    hasLastMessage(): boolean;
    clearLastMessage(): void;
    getLastMessage(): Message | undefined;
    setLastMessage(value?: Message): void;


    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Venue.AsObject;
    static toObject(includeInstance: boolean, msg: Venue): Venue.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Venue, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Venue;
    static deserializeBinaryFromReader(message: Venue, reader: jspb.BinaryReader): Venue;
}

export namespace Venue {
    export type AsObject = {
        origin?: Origin.AsObject,
        lastMessage?: Message.AsObject,
    }
}
