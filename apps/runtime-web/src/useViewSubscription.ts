import { useEffect, useState } from "react";
import type { View } from "@openwebhmi/protocol";
import type { GatewayClient } from "./gatewayClient";

type ViewState = {
  view: View | null;
  version: number | null;
  error: string | null;
};

export function useViewSubscription(
  client: GatewayClient,
  projectId: string,
  viewId: string,
): ViewState {
  const [state, setState] = useState<ViewState>({
    view: null,
    version: null,
    error: null,
  });

  useEffect(() => {
    setState({ view: null, version: null, error: null });

    const closeView = client.openView(projectId, viewId, (definition) => {
      setState({
        view: definition.view,
        version: definition.version,
        error: null,
      });
    });

    const unsubscribeProject = client.subscribeProject(projectId, (change) => {
      if (
        change.project_id !== projectId ||
        change.action === "deleted" ||
        change.artifact.kind !== "view" ||
        change.artifact.id !== viewId
      ) {
        return;
      }
      client.requestView(projectId, viewId);
    });

    const unsubscribeError = client.onError((error) => {
      if (error.code === "view.not_found") {
        setState((current) => ({
          ...current,
          error: `View "${viewId}" was not found in project "${projectId}".`,
        }));
      }
    });

    return () => {
      closeView();
      unsubscribeProject();
      unsubscribeError();
    };
  }, [client, projectId, viewId]);

  return state;
}
