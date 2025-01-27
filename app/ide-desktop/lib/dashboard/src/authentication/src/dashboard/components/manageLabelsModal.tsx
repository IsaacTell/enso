/** @file A modal to select labels for an asset. */
import * as React from 'react'

import * as auth from '../../authentication/providers/auth'
import * as backendModule from '../backend'
import * as backendProvider from '../../providers/backend'
import * as hooks from '../../hooks'
import * as modalProvider from '../../providers/modal'
import * as string from '../../string'

import ColorPicker from './colorPicker'
import Label from './label'
import Modal from './modal'

// =========================
// === ManageLabelsModal ===
// =========================

/** Props for a {@link ManageLabelsModal}. */
export interface ManageLabelsModalProps<
    Asset extends backendModule.AnyAsset = backendModule.AnyAsset,
> {
    item: Asset
    setItem: React.Dispatch<React.SetStateAction<Asset>>
    allLabels: Map<backendModule.LabelName, backendModule.Label>
    doCreateLabel: (value: string, color: backendModule.LChColor) => Promise<void>
    /** If this is `null`, this modal will be centered. */
    eventTarget: HTMLElement | null
}

/** A modal to select labels for an asset.
 * @throws {Error} when the current backend is the local backend, or when the user is offline.
 * This should never happen, as this modal should not be accessible in either case. */
export default function ManageLabelsModal<
    Asset extends backendModule.AnyAsset = backendModule.AnyAsset,
>(props: ManageLabelsModalProps<Asset>) {
    const { item, setItem, allLabels, doCreateLabel, eventTarget } = props
    const { organization } = auth.useNonPartialUserSession()
    const { backend } = backendProvider.useBackend()
    const { unsetModal } = modalProvider.useSetModal()
    const toastAndLog = hooks.useToastAndLog()
    const [labels, setLabels] = React.useState(item.labels ?? [])
    const [query, setQuery] = React.useState('')
    const [color, setColor] = React.useState<backendModule.LChColor | null>(null)
    const position = React.useMemo(() => eventTarget?.getBoundingClientRect(), [eventTarget])
    const labelNames = React.useMemo(() => new Set(labels), [labels])
    const regex = React.useMemo(() => new RegExp(string.regexEscape(query), 'i'), [query])
    const canSelectColor = React.useMemo(
        () =>
            query !== '' &&
            Array.from(allLabels.keys()).filter(label => regex.test(label)).length === 0,
        [allLabels, query, regex]
    )
    const canCreateNewLabel = canSelectColor && color != null

    React.useEffect(() => {
        setItem(oldItem => ({ ...oldItem, labels }))
    }, [labels, /* should never change */ setItem])

    if (backend.type === backendModule.BackendType.local || organization == null) {
        // This should never happen - the local backend does not have the "labels" column,
        // and `organization` is absent only when offline - in which case the user should only
        // be able to access the local backend.
        // This MUST be an error, otherwise the hooks below are considered as conditionally called.
        throw new Error('Cannot add labels to assets on the local backend.')
    } else {
        const doToggleLabel = React.useCallback(
            async (name: backendModule.LabelName) => {
                const newLabels = labelNames.has(name)
                    ? labels.filter(label => label !== name)
                    : [...labels, name]
                setLabels(newLabels)
                try {
                    await backend.associateTag(item.id, newLabels, item.title)
                } catch (error) {
                    toastAndLog(null, error)
                    setLabels(labels)
                }
            },
            [
                labelNames,
                labels,
                item.id,
                item.title,
                backend,
                /* should never change */ toastAndLog,
            ]
        )

        return (
            <Modal
                centered={eventTarget == null}
                className="absolute overflow-hidden bg-dim w-full h-full top-0 left-0 z-1"
            >
                <div
                    tabIndex={-1}
                    style={
                        position != null
                            ? {
                                  left: position.left + window.scrollX,
                                  top: position.top + window.scrollY,
                              }
                            : {}
                    }
                    className="sticky w-60"
                    onClick={mouseEvent => {
                        mouseEvent.stopPropagation()
                    }}
                    onContextMenu={mouseEvent => {
                        mouseEvent.stopPropagation()
                        mouseEvent.preventDefault()
                    }}
                    onKeyDown={event => {
                        if (event.key !== 'Escape') {
                            event.stopPropagation()
                        }
                    }}
                >
                    <div className="absolute bg-frame-selected backdrop-blur-3xl rounded-2xl h-full w-full" />
                    <form
                        className="relative flex flex-col gap-1 rounded-2xl gap-2 p-2"
                        onSubmit={async event => {
                            event.preventDefault()
                            setLabels(oldLabels => [...oldLabels, backendModule.LabelName(query)])
                            try {
                                if (color != null) {
                                    await doCreateLabel(query, color)
                                }
                            } catch (error) {
                                toastAndLog(null, error)
                                setLabels(oldLabels =>
                                    oldLabels.filter(oldLabel => oldLabel !== query)
                                )
                            }
                            unsetModal()
                        }}
                    >
                        <div>
                            <h2 className="text-sm font-bold">Labels</h2>
                            {/* Space reserved for other tabs. */}
                        </div>
                        <div
                            className={`flex items-center grow rounded-full border border-black-a10 gap-2 px-1 ${
                                // eslint-disable-next-line @typescript-eslint/no-magic-numbers
                                canSelectColor && color != null && color.lightness <= 50
                                    ? 'text-tag-text placeholder-tag-text'
                                    : 'text-primary'
                            }`}
                            style={
                                !canSelectColor || color == null
                                    ? {}
                                    : {
                                          backgroundColor: backendModule.lChColorToCssColor(color),
                                      }
                            }
                        >
                            <input
                                autoFocus
                                type="text"
                                placeholder="Type labels to search"
                                className="grow bg-transparent leading-170 h-6 px-1 py-px"
                                onChange={event => {
                                    setQuery(event.currentTarget.value)
                                }}
                            />
                        </div>
                        <button
                            type="submit"
                            disabled={!canCreateNewLabel}
                            className="text-tag-text bg-invite rounded-full px-2 py-1 disabled:opacity-30"
                        >
                            <div className="h-6 py-0.5">Create</div>
                        </button>
                        {canSelectColor && (
                            <div className="flex gap-1">
                                <div className="grow flex items-center gap-1">
                                    <ColorPicker setColor={setColor} />
                                </div>
                            </div>
                        )}
                        <div className="overflow-auto pl-1 pr-12 max-h-80">
                            {Array.from(allLabels.values())
                                .filter(label => regex.test(label.value))
                                .map(label => (
                                    <div key={label.id} className="flex items-center h-8">
                                        <Label
                                            active={labels.includes(label.value)}
                                            color={label.color}
                                            onClick={async () => {
                                                await doToggleLabel(label.value)
                                            }}
                                        >
                                            {label.value}
                                        </Label>
                                    </div>
                                ))}
                        </div>
                    </form>
                </div>
            </Modal>
        )
    }
}
