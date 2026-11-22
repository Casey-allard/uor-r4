// prism-prism replacement for the Functional Source License Annotator. Not Prism's code.
// The renderer opens this over a preview screenshot; here it hands the screenshot back as one
// attachment, unannotated. Drawing on the screenshot is a later module.
import type { FC } from "react";

export interface AnnotatorProps {
  screenshotUrl: string | null;
  onSubmit: (files: File[]) => void;
  handleAnnotatorClick: () => void;
}

export const Annotator: FC<AnnotatorProps> = ({ screenshotUrl, onSubmit, handleAnnotatorClick }) => {
  const submit = async () => {
    if (screenshotUrl) {
      const blob = await (await fetch(screenshotUrl)).blob();
      onSubmit([new File([blob], "screenshot.png", { type: blob.type || "image/png" })]);
    }
    handleAnnotatorClick();
  };
  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-4 p-4">
      {screenshotUrl ? <img src={screenshotUrl} alt="" className="max-h-[70%] max-w-full rounded-md border" /> : null}
      <div className="flex gap-2">
        <button type="button" className="rounded-md border px-3 py-1.5 text-sm" onClick={handleAnnotatorClick}>Back</button>
        <button type="button" className="rounded-md bg-foreground px-3 py-1.5 text-sm text-background" onClick={submit}>Attach screenshot</button>
      </div>
    </div>
  );
};
