import { useEffect, useMemo, useState } from "react";
import type { BoundValue } from "@openwebhmi/component-library";
import type { TagValue } from "@openwebhmi/protocol";
import type { GatewayClient } from "./gatewayClient";

export function useTagBindings(client: GatewayClient, paths: string[]) {
  const stableKey = useMemo(() => pathsKey(paths), [paths]);
  const stablePaths = useMemo(
    () => (stableKey.length === 0 ? [] : stableKey.split("\n")),
    [stableKey],
  );
  const [boundValues, setBoundValues] = useState<
    Record<string, BoundValue | undefined>
  >({});

  useEffect(() => {
    const allowed = new Set(stablePaths);
    setBoundValues((current) => {
      const next: Record<string, BoundValue | undefined> = {};
      for (const path of stablePaths) {
        next[path] = current[path];
      }
      return next;
    });

    const unsubscribers = stablePaths.map((path) =>
      client.subscribe(path, (update) => {
        if (!allowed.has(path)) {
          return;
        }
        setBoundValues((current) => ({
          ...current,
          [path]: update,
        }));
      }),
    );

    return () => {
      for (const unsubscribe of unsubscribers) {
        unsubscribe();
      }
    };
  }, [client, stableKey]);

  const writeTag = (path: string, value: TagValue) => {
    client.writeTag(path, value);
  };

  return { boundValues, writeTag };
}

function pathsKey(paths: string[]): string {
  return [...new Set(paths)].sort().join("\n");
}
