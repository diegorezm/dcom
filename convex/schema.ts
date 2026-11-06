import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  users: defineTable({
    tokenIdentifier: v.string(),
    displayName: v.string(),
    avatarId: v.optional(v.id("_storage")),
    lastSeenAt: v.number(),
  }).index("by_tokenIdentifier", ["tokenIdentifier"]),

  servers: defineTable({
    name: v.string(),
    iconId: v.optional(v.id("_storage")),
    ownerId: v.id("users"),
  }).index("by_ownerId", ["ownerId"]),

  members: defineTable({
    serverId: v.id("servers"),
    userId: v.id("users"),
    role: v.union(v.literal("owner"), v.literal("admin"), v.literal("member")),
    joinedAt: v.number(),
  })
    .index("by_serverId", ["serverId"])
    .index("by_userId", ["userId"])
    .index("by_serverId_userId", ["serverId", "userId"]),

  channels: defineTable({
    serverId: v.optional(v.id("servers")),
    name: v.string(),
    kind: v.union(v.literal("text"), v.literal("dm")),
    position: v.number(),
  }).index("by_serverId_position", ["serverId", "position"]),

  channelParticipants: defineTable({
    channelId: v.id("channels"),
    userId: v.id("users"),
  })
    .index("by_userId", ["userId"])
    .index("by_channelId", ["channelId"]),

  // O Convex acrescenta _creationTime ao fim de todo índice.
  messages: defineTable({
    channelId: v.id("channels"),
    authorId: v.id("users"),
    body: v.string(),
    mentions: v.array(v.id("users")),
    editedAt: v.optional(v.number()),
    deletedAt: v.optional(v.number()),
  }).index("by_channelId", ["channelId"]),

  readStates: defineTable({
    userId: v.id("users"),
    channelId: v.id("channels"),
    lastReadMessageId: v.optional(v.id("messages")),
  }).index("by_userId_channelId", ["userId", "channelId"]),

  invites: defineTable({
    code: v.string(),
    serverId: v.id("servers"),
    createdBy: v.id("users"),
    expiresAt: v.optional(v.number()),
    maxUses: v.optional(v.number()),
    uses: v.number(),
  }).index("by_code", ["code"]),
});
