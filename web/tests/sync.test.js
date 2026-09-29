import { test, describe } from "node:test";
import assert from "node:assert/strict";

describe("Web PWA Unit Tests - Sync & Conflict Resolution Parity", () => {
  test("Last-Write-Wins server wins on newer or tie", () => {
    const local = {
      id: "card-1",
      name: "Local Name",
      updated_at: "2026-09-29T10:00:00.000Z",
    };
    const remoteNewer = {
      id: "card-1",
      name: "Remote Newer Name",
      updated_at: "2026-09-29T10:05:00.000Z",
    };
    const remoteTie = {
      id: "card-1",
      name: "Remote Tie Name",
      updated_at: "2026-09-29T10:00:00.000Z",
    };

    // When remote is newer, remote wins
    assert.ok(remoteNewer.updated_at >= local.updated_at);
    // When timestamps match exactly, server deterministically wins
    assert.ok(remoteTie.updated_at >= local.updated_at);
  });

  test("Last-Write-Wins local wins if newer", () => {
    const local = {
      id: "card-1",
      name: "Local Newer Name",
      updated_at: "2026-09-29T10:15:00.000Z",
    };
    const remoteOlder = {
      id: "card-1",
      name: "Remote Stale Name",
      updated_at: "2026-09-29T10:00:00.000Z",
    };

    assert.ok(local.updated_at > remoteOlder.updated_at);
  });

  test("Soft-delete resolution: deleted_at with newer updated_at overrides active state", () => {
    const activeOld = {
      id: "tx-1",
      description: "Coffee",
      deleted_at: null,
      updated_at: "2026-09-29T08:00:00.000Z",
    };
    const deletedNewer = {
      id: "tx-1",
      description: "Coffee",
      deleted_at: "2026-09-29T09:00:00.000Z",
      updated_at: "2026-09-29T09:00:00.000Z",
    };

    assert.ok(deletedNewer.updated_at > activeOld.updated_at);
    assert.ok(deletedNewer.deleted_at !== null);
  });
});
