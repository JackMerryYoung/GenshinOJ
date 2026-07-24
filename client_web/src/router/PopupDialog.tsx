import { MouseEventHandler } from "react";
import { useTranslation } from "react-i18next";

import {
    Button,
    Dialog,
    DialogBody,
    DialogContent,
    DialogTrigger,
    DialogSurface,
    DialogActions,
    DialogOpenChangeEvent,
    DialogOpenChangeData,
} from "@fluentui/react-components";

export default function PopupDialog({ text, open, setPopupDialogOpenState, onClose }: {
    text: string,
    open: boolean,
    setPopupDialogOpenState: React.Dispatch<React.SetStateAction<boolean>>,
    onClose?: MouseEventHandler<HTMLButtonElement> | undefined,
}) {
    const { t } = useTranslation("common");
    const handleClose = (event: React.MouseEvent<HTMLButtonElement>) => {
        if (onClose) onClose(event);
        setPopupDialogOpenState(false);
    };

    const handleOpenChange = (_: DialogOpenChangeEvent, data: DialogOpenChangeData) => {
        setPopupDialogOpenState(data.open);
    };

    return <Dialog modalType="alert" open={open} onOpenChange={handleOpenChange}>
        <DialogSurface>
            <DialogBody>
                <DialogContent>{text}</DialogContent>
                <DialogActions>
                    <DialogTrigger disableButtonEnhancement>
                        <Button appearance="primary" onClick={handleClose}>{t("action.close")}</Button>
                    </DialogTrigger>
                </DialogActions>
            </DialogBody>
        </DialogSurface>
    </Dialog>
}