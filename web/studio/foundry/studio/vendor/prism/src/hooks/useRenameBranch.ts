import { useMutation, useQueryClient } from "@tanstack/react-query";
import { ipc } from "@/ipc/types";
import { showError } from "@/lib/toast";
import { selectedAppIdAtom } from "@/atoms/appAtoms";
import { useAtomValue } from "jotai";
import { queryKeys } from "@/lib/queryKeys";
import { PrismError, PrismErrorKind } from "@/errors/prism_error";

interface RenameBranchParams {
  appId: number;
  oldBranchName: string;
  newBranchName: string;
}

export function useRenameBranch() {
  const queryClient = useQueryClient();
  const currentAppId = useAtomValue(selectedAppIdAtom);

  const mutation = useMutation<void, Error, RenameBranchParams>({
    mutationFn: async (params: RenameBranchParams) => {
      if (params.appId === null || params.appId === undefined) {
        throw new PrismError(
          "App ID is required to rename a branch.",
          PrismErrorKind.Validation,
        );
      }
      if (!params.oldBranchName) {
        throw new PrismError(
          "Old branch name is required.",
          PrismErrorKind.Validation,
        );
      }
      if (!params.newBranchName) {
        throw new PrismError(
          "New branch name is required.",
          PrismErrorKind.Validation,
        );
      }
      await ipc.app.renameBranch(params);
    },
    onSuccess: (_, variables) => {
      // Invalidate queries that depend on branch information
      queryClient.invalidateQueries({
        queryKey: queryKeys.branches.byApp({ appId: variables.appId }),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.versions.list({ appId: variables.appId }),
      });
      // Potentially show a success message or trigger other actions
    },
    meta: {
      showErrorToast: true,
    },
  });

  const renameBranch = async (params: Omit<RenameBranchParams, "appId">) => {
    if (!currentAppId) {
      showError("No application selected.");
      return;
    }
    return mutation.mutateAsync({ ...params, appId: currentAppId });
  };

  return {
    renameBranch,
    isRenamingBranch: mutation.isPending,
    renameBranchError: mutation.error,
  };
}
