import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { useMemo, useRef, useState } from "react";

import { libraryKeys } from "@/modules/library/api/keys";
import { unwrapForQuery } from "@/utils/query";

export const Route = createFileRoute("/browse")({
  component: BrowsePage,
});

interface BrowseSkin {
  champion: string;
  name: string;
  path: string;
}

type RowState = { kind: "installing" } | { kind: "done" } | { kind: "error"; message: string };

type IpcResultLike = Parameters<typeof unwrapForQuery>[0];

const MAX_ROWS = 150;
const REPO_URL = "https://github.com/bettie9/LeagueSkins";

function errorMessage(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) {
    return String((e as { message: unknown }).message);
  }
  return "Something went wrong";
}

async function fetchListing(force: boolean): Promise<BrowseSkin[]> {
  const result = await invoke("browse_list_skins", { force });
  return unwrapForQuery(result as IpcResultLike) as BrowseSkin[];
}

function BrowsePage() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [rows, setRows] = useState<Record<string, RowState>>({});
  const forceRef = useRef(false);

  const listing = useQuery({
    queryKey: ["browse", "skins"],
    staleTime: 30 * 60 * 1000,
    queryFn: () => {
      const force = forceRef.current;
      forceRef.current = false;
      return fetchListing(force);
    },
  });

  const filtered = useMemo(() => {
    const all = listing.data ?? [];
    const q = search.trim().toLowerCase();
    if (!q) return all;
    return all.filter(
      (s) => s.name.toLowerCase().includes(q) || s.champion.toLowerCase().includes(q),
    );
  }, [listing.data, search]);

  const shown = filtered.slice(0, MAX_ROWS);
  const busy = Object.values(rows).some((r) => r.kind === "installing");

  async function install(skin: BrowseSkin) {
    setRows((r) => ({ ...r, [skin.path]: { kind: "installing" } }));
    try {
      const result = await invoke("browse_install_skin", { path: skin.path });
      unwrapForQuery(result as IpcResultLike);
      setRows((r) => ({ ...r, [skin.path]: { kind: "done" } }));
      void queryClient.invalidateQueries({ queryKey: libraryKeys.mods() });
    } catch (e) {
      setRows((r) => ({ ...r, [skin.path]: { kind: "error", message: errorMessage(e) } }));
    }
  }

  return (
    <div className="h-full overflow-y-auto p-4">
      <div className="mx-auto flex max-w-3xl flex-col gap-3">
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search skins or champions..."
            className="w-full rounded-md border border-surface-700 bg-transparent px-3 py-2 text-sm text-surface-200"
          />
          <button
            type="button"
            disabled={listing.isFetching}
            onClick={() => {
              forceRef.current = true;
              void listing.refetch();
            }}
            className="rounded-md border border-surface-700 px-3 py-2 text-sm text-surface-200 disabled:opacity-50"
          >
            Refresh
          </button>
        </div>

        {listing.isLoading && <p className="text-sm text-surface-400">Loading skin list...</p>}
        {listing.isError && (
          <p className="text-sm text-red-400">
            Could not load the skin list: {errorMessage(listing.error)}
          </p>
        )}
        {listing.data && (
          <p className="text-xs text-surface-400">
            {filtered.length} skins
            {filtered.length > MAX_ROWS
              ? ` (showing the first ${MAX_ROWS}, search to narrow down)`
              : ""}
            . Stop the patcher before installing.
          </p>
        )}

        <ul className="flex flex-col gap-2">
          {shown.map((skin) => {
            const state = rows[skin.path];
            return (
              <li
                key={skin.path}
                className="flex items-center justify-between gap-3 rounded-md border border-surface-700 px-3 py-2"
              >
                <div className="min-w-0">
                  <p className="truncate text-sm text-surface-200">{skin.name}</p>
                  <p className="truncate text-xs text-surface-400">{skin.champion}</p>
                  {state?.kind === "error" && (
                    <p className="text-xs text-red-400">{state.message}</p>
                  )}
                </div>
                <button
                  type="button"
                  disabled={busy || state?.kind === "done"}
                  onClick={() => void install(skin)}
                  className="shrink-0 rounded-md bg-accent-500 px-3 py-1.5 text-sm text-white disabled:opacity-50"
                >
                  {state?.kind === "installing"
                    ? "Installing..."
                    : state?.kind === "done"
                      ? "Installed"
                      : "Install"}
                </button>
              </li>
            );
          })}
        </ul>

        <p className="pt-2 text-xs text-surface-400">
          Skins come from the community mirror{" "}
          <button type="button" className="underline" onClick={() => void open(REPO_URL)}>
            bettie9/LeagueSkins
          </button>
          . Not affiliated with its author, the LeagueToolkit team or Riot Games.
        </p>
      </div>
    </div>
  );
}
