/** @file The icon and name of a {@link backendModule.DirectoryAsset}. */
import * as React from 'react'

import FolderIcon from 'enso-assets/folder.svg'
import TriangleDownIcon from 'enso-assets/triangle_down.svg'

import * as assetEventModule from '../events/assetEvent'
import * as assetListEventModule from '../events/assetListEvent'
import * as assetTreeNode from '../assetTreeNode'
import * as backendModule from '../backend'
import * as backendProvider from '../../providers/backend'
import type * as column from '../column'
import * as eventModule from '../event'
import * as hooks from '../../hooks'
import * as indent from '../indent'
import * as shortcutsModule from '../shortcuts'
import * as shortcutsProvider from '../../providers/shortcuts'
import * as visibility from '../visibility'

import EditableSpan from './editableSpan'
import SvgMask from '../../authentication/components/svgMask'

// =====================
// === DirectoryName ===
// =====================

/** Props for a {@link DirectoryNameColumn}. */
export interface DirectoryNameColumnProps extends column.AssetColumnProps {}

/** The icon and name of a {@link backendModule.DirectoryAsset}.
 * @throws {Error} when the asset is not a {@link backendModule.DirectoryAsset}.
 * This should never happen. */
export default function DirectoryNameColumn(props: DirectoryNameColumnProps) {
    const {
        item,
        setItem,
        selected,
        setSelected,
        state: {
            numberOfSelectedItems,
            assetEvents,
            dispatchAssetListEvent,
            topLevelAssets,
            nodeMap,
            doToggleDirectoryExpansion,
        },
        rowState,
        setRowState,
    } = props
    const toastAndLog = hooks.useToastAndLog()
    const { backend } = backendProvider.useBackend()
    const { shortcuts } = shortcutsProvider.useShortcuts()
    const [isHovered, setIsHovered] = React.useState(false)
    const [shouldAnimate, setShouldAnimate] = React.useState(false)
    const asset = item.item
    if (asset.type !== backendModule.AssetType.directory) {
        // eslint-disable-next-line no-restricted-syntax
        throw new Error('`DirectoryNameColumn` can only display directory assets.')
    }
    const setAsset = assetTreeNode.useSetAsset(asset, setItem)

    React.useEffect(() => {
        if (isHovered) {
            // Delay adding animation CSS attributes, to prevent animations for
            // the initial hover.
            requestAnimationFrame(() => {
                setShouldAnimate(true)
            })
        } else {
            setShouldAnimate(false)
        }
    }, [isHovered])

    const doRename = async (newName: string) => {
        if (backend.type !== backendModule.BackendType.local) {
            try {
                await backend.updateDirectory(asset.id, { title: newName }, asset.title)
                return
            } catch (error) {
                toastAndLog('Could not rename folder', error)
                throw error
            }
        }
    }

    hooks.useEventHandler(assetEvents, async event => {
        switch (event.type) {
            case assetEventModule.AssetEventType.newProject:
            case assetEventModule.AssetEventType.uploadFiles:
            case assetEventModule.AssetEventType.newDataConnector:
            case assetEventModule.AssetEventType.openProject:
            case assetEventModule.AssetEventType.closeProject:
            case assetEventModule.AssetEventType.cancelOpeningAllProjects:
            case assetEventModule.AssetEventType.cut:
            case assetEventModule.AssetEventType.cancelCut:
            case assetEventModule.AssetEventType.move:
            case assetEventModule.AssetEventType.delete:
            case assetEventModule.AssetEventType.restore:
            case assetEventModule.AssetEventType.downloadSelected:
            case assetEventModule.AssetEventType.removeSelf:
            case assetEventModule.AssetEventType.temporarilyAddLabels:
            case assetEventModule.AssetEventType.temporarilyRemoveLabels:
            case assetEventModule.AssetEventType.addLabels:
            case assetEventModule.AssetEventType.removeLabels:
            case assetEventModule.AssetEventType.deleteLabel: {
                // Ignored. These events should all be unrelated to directories.
                // `deleteMultiple`, `restoreMultiple` and `downloadSelected` are handled by
                // `AssetRow`.
                break
            }
            case assetEventModule.AssetEventType.newFolder: {
                if (item.key === event.placeholderId) {
                    if (backend.type !== backendModule.BackendType.remote) {
                        toastAndLog('Cannot create folders on the local drive')
                    } else {
                        rowState.setVisibility(visibility.Visibility.faded)
                        try {
                            const createdDirectory = await backend.createDirectory({
                                parentId: asset.parentId,
                                title: asset.title,
                            })
                            rowState.setVisibility(visibility.Visibility.visible)
                            setAsset({
                                ...asset,
                                ...createdDirectory,
                            })
                        } catch (error) {
                            dispatchAssetListEvent({
                                type: assetListEventModule.AssetListEventType.delete,
                                key: item.key,
                            })
                            toastAndLog('Could not create new folder', error)
                        }
                    }
                }
                break
            }
        }
    })

    return (
        <div
            className={`flex text-left items-center whitespace-nowrap rounded-l-full gap-1 px-1.5 py-1 min-w-max ${indent.indentClass(
                item.depth
            )}`}
            onMouseEnter={() => {
                setIsHovered(true)
            }}
            onMouseLeave={() => {
                setIsHovered(false)
            }}
            onKeyDown={event => {
                if (rowState.isEditingName && event.key === 'Enter') {
                    event.stopPropagation()
                }
            }}
            onClick={event => {
                if (
                    eventModule.isSingleClick(event) &&
                    ((selected && numberOfSelectedItems === 1) ||
                        shortcuts.matchesMouseAction(shortcutsModule.MouseAction.editName, event))
                ) {
                    setRowState(oldRowState => ({
                        ...oldRowState,
                        isEditingName: true,
                    }))
                } else if (eventModule.isDoubleClick(event)) {
                    if (!rowState.isEditingName) {
                        // This must be processed on the next tick, otherwise it will be overridden
                        // by the default click handler.
                        window.setTimeout(() => {
                            setSelected(false)
                        }, 0)
                        doToggleDirectoryExpansion(asset.id, item.key, asset.title)
                    }
                }
            }}
        >
            {isHovered ? (
                <SvgMask
                    src={TriangleDownIcon}
                    className={`cursor-pointer h-4 w-4 m-1 ${
                        shouldAnimate ? 'transition-transform duration-300' : ''
                    } ${item.children != null ? '' : '-rotate-90'}`}
                    onClick={event => {
                        event.stopPropagation()
                        doToggleDirectoryExpansion(asset.id, item.key, asset.title)
                    }}
                />
            ) : (
                <SvgMask src={FolderIcon} className="h-4 w-4 m-1" />
            )}
            <EditableSpan
                editable={rowState.isEditingName}
                checkSubmittable={newTitle =>
                    (item.directoryKey != null
                        ? nodeMap.current.get(item.directoryKey)?.children ?? []
                        : topLevelAssets.current
                    ).every(
                        child =>
                            // All siblings,
                            child.key === item.key ||
                            // that are directories,
                            !backendModule.assetIsDirectory(child.item) ||
                            // must have a different name.
                            child.item.title !== newTitle
                    )
                }
                onSubmit={async newTitle => {
                    setRowState(oldRowState => ({
                        ...oldRowState,
                        isEditingName: false,
                    }))
                    if (newTitle !== asset.title) {
                        const oldTitle = asset.title
                        setAsset(oldItem => ({ ...oldItem, title: newTitle }))
                        try {
                            await doRename(newTitle)
                        } catch {
                            setAsset(oldItem => ({ ...oldItem, title: oldTitle }))
                        }
                    }
                }}
                onCancel={() => {
                    setRowState(oldRowState => ({
                        ...oldRowState,
                        isEditingName: false,
                    }))
                }}
                className={`cursor-pointer bg-transparent grow leading-170 h-6 py-px ${
                    rowState.isEditingName ? 'cursor-text' : 'cursor-pointer'
                }`}
            >
                {asset.title}
            </EditableSpan>
        </div>
    )
}
