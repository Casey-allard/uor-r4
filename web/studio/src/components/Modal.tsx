import { useEffect, useId, useRef } from 'preact/hooks';
import type { ComponentChildren } from 'preact';

interface ModalProps {
    title: string;
    onClose: () => void;
    children: ComponentChildren;
}

export function Modal({ title, onClose, children }: ModalProps) {
    const ref = useRef<HTMLDialogElement>(null);
    const titleId = useId();
    const onCloseRef = useRef(onClose);
    onCloseRef.current = onClose;

    useEffect(() => {
        const dialog = ref.current;
        if (!dialog) return;
        const closed = () => onCloseRef.current();
        dialog.addEventListener('close', closed);
        dialog.showModal();
        return () => {
            dialog.removeEventListener('close', closed);
            if (dialog.open) dialog.close();
        };
    }, []);

    return (
        <dialog ref={ref} aria-labelledby={titleId}>
            <h2 id={titleId}>{title}</h2>
            {children}
            <form method="dialog">
                <button type="submit" value="close">Close</button>
            </form>
        </dialog>
    );
}
