import { useQuery } from "@tanstack/react-query";
import { ipc, type ProblemReport } from "@/ipc/types";
import { queryKeys } from "@/lib/queryKeys";
import { PrismError, PrismErrorKind } from "@/errors/prism_error";

export function useCheckProblems(appId: number | null) {
  const {
    data: problemReport,
    isLoading: isChecking,
    error,
    refetch: checkProblems,
  } = useQuery<ProblemReport, Error>({
    queryKey: queryKeys.problems.byApp({ appId }),
    queryFn: async (): Promise<ProblemReport> => {
      if (!appId) {
        throw new PrismError("App ID is required", PrismErrorKind.Validation);
      }
      return ipc.misc.checkProblems({ appId });
    },
    enabled: false,
    // DO NOT SHOW ERROR TOAST.
  });

  return {
    problemReport,
    isChecking,
    error,
    checkProblems,
  };
}
