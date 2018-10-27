import * as grpc from 'grpc'
import * as matrix from 'matrix-js-sdk'
import * as timestamp_pb from 'google-protobuf/google/protobuf/timestamp_pb'
import * as protos from './haberdasher_pb'
import * as rpc from './haberdasher_grpc_pb'

require('dotenv').config()

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
    const client = new rpc.AgentSubscriberClient(
        '127.0.0.1:42253', grpc.credentials.createInsecure())
    const stream = client.publishVenueUpdates((error, response) => {
        console.log('stream resolved %s %s', error, response)
    })
    stream.on('event', (ev) => {
        console.log('stream %s', ev.getType())
    })
    console.log('stream ready')

    const matrixClient = matrix.createClient({
        baseUrl: 'https://matrix.org',
        accessToken: process.env.MATRIX_ACCESS_TOKEN,
        userId: process.env.MATRIX_USER_ID,
    })
    matrixClient.on('event', (ev) => {
        console.log('matrix %s', ev.getType())
    })
    matrixClient.on('Room.timeline', async (event, room, toStartOfTimeline) => {
        if (toStartOfTimeline || !room || event.getType() !== 'm.room.message') {
            return
        }

        const group = new protos.Group()
        group.setId(Buffer.from(room.roomId))
        group.setName(room.name)

        var sender = event.sender
        const individual = new protos.Individual()
        individual.setId(Buffer.from(sender.userId))
        individual.setName(sender.name)
        const performer = new protos.Performer()
        performer.setIndividual(individual)

        const at = new timestamp_pb.Timestamp()
        const atMs = event.getTs()
        at.setSeconds((atMs / 1_000) | 0)
        at.setNanos((atMs % 1_000) * 1_000_000)

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
        venue.setGroup(group)
        venue.setLastMessage(message)
        console.log('sending %s', JSON.stringify(venue.toObject()))

        stream.write(venue)
    })

    matrixClient.startClient({
        disablePresence: true,
        lazyLoadMembers: true,
    })
    //const res = await matrixClient.getDevices()
    //console.log('public rooms %s', JSON.stringify(res))
}

main()

process.on('uncaughtException', (err) => {
    console.log('process on uncaughtException error: %s', err)
})

process.on('unhandledRejection', (err) => {
    console.log('process on unhandledRejection error: %s', err)
})
