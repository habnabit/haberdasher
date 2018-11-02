import * as grpc from 'grpc'
import * as matrix from 'matrix-js-sdk'
import * as timestamp_pb from 'google-protobuf/google/protobuf/timestamp_pb'
import { fromEvent, fromEventPattern, Observable, Subscriber, combineLatest, empty, of } from 'rxjs'
import { filter, scan, map, startWith, tap, withLatestFrom, retryWhen, catchError } from 'rxjs/operators'

import * as protos from './haberdasher_pb'
import * as rpc from './haberdasher_grpc_pb'

require('dotenv').config()

const agentDescription = new protos.Agent()
agentDescription.setName('haberdasher_matrix_agent')

function isStatusObject(o: any): o is grpc.StatusObject {
    return 'code' in o && 'metadata' in o
}

function isServiceError(e: Error): e is grpc.ServiceError {
    return isStatusObject(e)
}

async function login() {
    const matrixClient = matrix.createClient('https://matrix.org')
    const res = await matrixClient.login('m.login.password', {
        identifier: {
            type: 'm.id.user',
            user: '',
        },
        password: '',
    })
}

async function main() {
    const meta = new grpc.Metadata()
    meta.add('access-token', process.env.HABERDASHER_ACCESS_TOKEN)

    const client = new rpc.AgentSubscriberClient(
        '127.0.0.1:42253', grpc.credentials.createInsecure())

    const venueStream$: Observable<grpc.ClientWritableStream<protos.Venue>> = Observable.create((observer: Subscriber<grpc.ClientWritableStream<protos.Venue>>) => {
        const stream = client.publishVenueUpdates(meta, (error, response) => {
            console.log('stream resolved %s/%s', error, response)
            observer.error(error || new Error('stream cleanly closed for unknown reasons'))
        })
        observer.next(stream)
        console.log('venue stream established')
    }).pipe(
        tap(
            (stream) => console.log('new stream %s', stream),
            (error) => console.log('stream error %s', error),
            () => console.log('stream complete')),
        retryWhen((notifier) => notifier.pipe(map((error) => {
            console.log('venue stream error: %s', error)
            if (isServiceError(error) && error.code === grpc.status.PERMISSION_DENIED) {
                throw error
            } else {
                return empty()
            }
        }))),
    )

    const matrixClient = matrix.createClient({
        baseUrl: 'https://matrix.org',
        accessToken: process.env.MATRIX_ACCESS_TOKEN,
        userId: process.env.MATRIX_USER_ID,
    })

    const roomIdToDmUserId$ = fromEvent(matrixClient, 'accountData').pipe(
        filter((ev: any) => ev.getType() === 'm.direct'),
        map((ev: any) => ev.getContent()),
        startWith({}),
        scan<{[userId: string]: string[]}, Map<string, string>>((roomIdToDmUserId, userIdToDmRoomId) => {
            Object.entries(userIdToDmRoomId).forEach(([userId, roomIds]) => {
                roomIds.forEach((roomId) => {
                    roomIdToDmUserId.set(roomId, userId)
                })
            })
            return roomIdToDmUserId
        }, new Map()),
    )

    const venueUpdates$ = fromEvent(matrixClient, 'Room.timeline').pipe(
        filter(([ev, room, toStartOfTimeline]) => room && !toStartOfTimeline && ev.getType() === 'm.room.message'),
        withLatestFrom(roomIdToDmUserId$),
        map(([[event, room], roomIdToDmUserId]) => {
            var sender = event.sender
            const performer = new protos.Performer()
            if (matrixClient.getUserId() === sender.userId) {
                performer.setMyself(true)
            } else {
                const individual = new protos.Individual()
                individual.setId(sender.userId)
                individual.setName(sender.name)
                performer.setIndividual(individual)
            }

            const at = new timestamp_pb.Timestamp()
            const atMs = event.getTs()
            at.setSeconds((atMs / 1_000) | 0)
            at.setNanos(((atMs % 1_000) | 0) * 1_000_000)

            const content = event.getContent()
            const message = new protos.Message()
            if (content.msgtype === 'm.emote') {
                message.setPose(content.body)
            } else {
                message.setText(content.body)
            }
            message.setPerformer(performer)
            message.setAt(at)

            const venue = new protos.Venue()
            venue.setLastMessage(message)

            const dmUserId = roomIdToDmUserId.get(room.roomId)
            if (dmUserId) {
                const dmIndividual = new protos.Individual()
                dmIndividual.setId(dmUserId)
                const dmUser = matrixClient.getUser(dmUserId)
                if (dmUser) {
                    dmIndividual.setName(dmUser.displayName)
                }
                venue.setIndividual(dmIndividual)
            } else {
                const group = new protos.Group()
                group.setId(room.roomId)
                group.setName(room.name)
                venue.setGroup(group)
            }
            return venue
        }),
    )

    venueUpdates$.pipe(
        withLatestFrom(venueStream$),
    ).subscribe(([venue, stream]) => {
        console.log('subscription %s <- %s', stream, JSON.stringify(venue.toObject()))
        stream.write(venue)
    })

    matrixClient.startClient({
        disablePresence: true,
        lazyLoadMembers: true,
    })
}

main()

process.on('uncaughtException', (err) => {
    console.log('process on uncaughtException error: %s', err)
})

process.on('unhandledRejection', (err) => {
    console.log('process on unhandledRejection error: %s', err)
})
