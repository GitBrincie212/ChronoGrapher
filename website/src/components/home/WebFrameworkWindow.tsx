/** biome-ignore-all lint/a11y/noSvgWithoutTitle: Shut up */

import React, {type JSX, useEffect} from "react";
import {animate, createTimeline} from "animejs";

function WebFrameworkBrowserWindowStoplightDot(props: {
    type: "minimize" | "maximize" | "delete";
    isStoplightHovered: boolean;
    onClick: React.MouseEventHandler<HTMLButtonElement>;
}) {
    return (
        <button
            type={"button"}
            onClick={props.onClick}
            className={`rounded-full h-2 min-w-2 text-[0.3rem] flex items-center justify-center ${
                props.type === "minimize"
                    ? "bg-fd-warning text-[#5c4115]"
                    : props.type === "maximize"
                        ? "bg-fd-success text-[#024f21]"
                        : "bg-fd-error text-[#451012]"
            }`}
        >
            {props.isStoplightHovered &&
                (props.type === "minimize" ? (
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="1.4em"
                        height="1.4em"
                        viewBox="0 0 12 12"
                    >
                        <path
                            fill="currentColor"
                            d="M2 6a.75.75 0 0 1 .75-.75h6.5a.75.75 0 0 1 0 1.5h-6.5A.75.75 0 0 1 2 6"
                        />
                    </svg>
                ) : props.type === "maximize" ? (
                    <svg
                        width="1em"
                        height="1em"
                        viewBox="0 0 16 16"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path d="M15 3.70703V15H3.70703L15 3.70703Z" fill="currentColor" />
                        <path d="M12.293 1L1 12.293V1H12.293Z" fill="currentColor" />
                    </svg>
                ) : (
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="1em"
                        height="1em"
                        viewBox="0 0 15 15"
                    >
                        <path
                            fill="currentColor"
                            d="M3.64 2.27L7.5 6.13l3.84-3.84A.92.92 0 0 1 12 2a1 1 0 0 1 1 1a.9.9 0 0 1-.27.66L8.84 7.5l3.89 3.89A.9.9 0 0 1 13 12a1 1 0 0 1-1 1a.92.92 0 0 1-.69-.27L7.5 8.87l-3.85 3.85A.92.92 0 0 1 3 13a1 1 0 0 1-1-1a.9.9 0 0 1 .27-.66L6.16 7.5L2.27 3.61A.9.9 0 0 1 2 3a1 1 0 0 1 1-1c.24.003.47.1.64.27"
                        />
                    </svg>
                ))}
        </button>
    );
}

export default function WebFrameworkBrowserWindow(props: {
    titles: string[];
    icons: JSX.Element[];
    activeColors: [string, string][];
    position: [number, number];
}) {
    const activeBackground = React.useRef<HTMLDivElement>(null);
    const activeIcon = React.useRef<HTMLDivElement>(null);
    const windowContainer = React.useRef<HTMLDivElement>(null);

    const [activeTabIndex, setActiveTabIndex] = React.useState<number>(0);
    const [isStoplightHovered, setIsStoplightHovered] =
        React.useState<boolean>(false);
    const [prevActiveIdx, setPrevActiveIdx] = React.useState<number>(0);
    const [beforeMaximizedContents, setBeforeMaximizedContents] = React.useState<{
        prevZ: string,
        prevPosX: string,
        prevPosY: string
    } | undefined>(undefined);

    // biome-ignore lint/correctness/useExhaustiveDependencies: Listing 'prevActiveIdx' will cause an infinite recursion
    useEffect(() => {
        if (!activeIcon.current || !activeBackground.current || prevActiveIdx === activeTabIndex) {
            return;
        }

        const timeline = createTimeline({
            duration: 50,
            playbackEase: "outBack",
        });

        const diff = activeTabIndex - prevActiveIdx;

        timeline
            .add(activeIcon.current, { x: diff > 0 ? 100 : -100, scale: 0 }, "+=0")
            .add(
                activeBackground.current,
                { background: props.activeColors[prevActiveIdx][0] },
                "<<",
            )
            .call(() => setPrevActiveIdx(activeTabIndex))
            .add(activeIcon.current, { x: [diff > 0 ? -100 : 100, 0], scale: 1 }, "+=0")
            .add(
                activeBackground.current,
                { background: props.activeColors[activeTabIndex][0] },
                "<<",
            );
    }, [activeTabIndex]);

    return (
        <div
            className={
                "z-10 absolute w-64 h-36 rounded-lg bg-fd-background overflow-hidden border border-fd-background"
            }
            style={{
                left: `${props.position[0]}rem`,
                top: `${props.position[1]}rem`,
                filter: "drop-shadow(0 0 2px black)",
            }}
            ref={windowContainer}
        >
            <div
                className={
                    "p-2 w-full flex gap-1 h-8 border-b border-b-fd-card-foreground items-center"
                }
            >
                {/** biome-ignore lint/a11y/noStaticElementInteractions: Simply visual falir, no interactivity present */}
                <div
                    className={"flex gap-1 h-full items-center"}
                    onMouseEnter={() => setIsStoplightHovered(true)}
                    onMouseLeave={() => setIsStoplightHovered(false)}
                >
                    <WebFrameworkBrowserWindowStoplightDot
                        type={"delete"}
                        isStoplightHovered={isStoplightHovered}
                        onClick={() => {}}
                    />
                    <WebFrameworkBrowserWindowStoplightDot
                        type={"minimize"}
                        isStoplightHovered={isStoplightHovered}
                        onClick={() => {}}
                    />
                    <WebFrameworkBrowserWindowStoplightDot
                        type={"maximize"}
                        isStoplightHovered={isStoplightHovered}
                        onClick={() => {
                            if (!windowContainer.current) {
                                console.error("windowContainer is undefined")
                                return;
                            }

                            if (beforeMaximizedContents) {
                                windowContainer.current.style.zIndex = beforeMaximizedContents.prevZ;

                                animate(windowContainer.current, {
                                    width: "16rem",
                                    height: "9rem",
                                    top: beforeMaximizedContents.prevPosY,
                                    left: beforeMaximizedContents.prevPosX,
                                    duration: 200,
                                    playbackEase: "linear"
                                })

                                setBeforeMaximizedContents(undefined);
                                return;
                            }

                            setBeforeMaximizedContents({
                                prevZ: windowContainer.current.style.zIndex,
                                prevPosX: windowContainer.current.style.left.slice(0, -3),
                                prevPosY: windowContainer.current.style.top.slice(0, -3)
                            });

                            windowContainer.current.style.zIndex = "1000";
                            animate(windowContainer.current, {
                                left: 0,
                                top: 0,
                                width: "100%",
                                height: "100%",
                                duration: 200,
                                playbackEase: "linear"
                            })
                        }}
                    />
                </div>
                <div className={"w-full flex gap-1"}>
                    {props.icons.map((icon, index) => (
                        <button
                            key={`tab-btn-${props.titles[index]}`}
                            type={"button"}
                            className={
                                "w-full h-4 select-none cursor-pointer rounded-md text-xs " +
                                `${index === activeTabIndex ? "bg-fd-background-100p" : "bg-transparent"}`
                            }
                            onClick={() => setActiveTabIndex(index)}
                        >
                            <div
                                className={
                                    "flex items-center gap-1 w-full h-full pl-1 py-0.5 font-[VioletSans]"
                                }
                            >
                                {icon}
                                <div className={"text-[0.5rem] cursor-pointer"}>
                                    {props.titles[index]}
                                </div>
                            </div>
                        </button>
                    ))}
                </div>
            </div>
            <div
                className={
                    "contents *:w-full *:h-full *:flex *:justify-center *:items-center *:pointer-events-none"
                }
            >
                <div
                    className={""}
                    style={{
                        background: props.activeColors[0][0],
                    }}
                    ref={activeBackground}
                />
                <div
                    className={"absolute inset-0 mt-4 text-6xl"}
                    style={{
                        color: props.activeColors[0][1],
                    }}
                    ref={activeIcon}
                >
                    {props.icons[prevActiveIdx]}
                </div>
            </div>
        </div>
    );
}