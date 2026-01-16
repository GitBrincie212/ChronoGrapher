/** biome-ignore-all lint/a11y/noSvgWithoutTitle: Yet again these are icons */
"use client";

import { createTimeline, waapi } from "animejs";
import Image from "next/image";
import React, { type JSX, useEffect } from "react";
import ChronoGrapherDark from "@/assets/chronographer-dark.svg";
import ChronoGrapherLight from "@/assets/chronographer-light.svg";
import { useFumadocsTheme } from "@/components/theme";
import styles from "./chronographer-wire-component.module.css";

import animate = waapi.animate;

const WIRES = [
  {
    widthClass: "w-[30rem] lg:w-[40rem] 2xl:w-[60rem]",
    leftIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <path d="M22 13V6a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v12c0 1.1.9 2 2 2h8" />
          <path d="m22 7l-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7m14 12l2 2l4-4" />
        </g>
      </svg>
    ),
    rightIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <ellipse cx="12" cy="5" rx="9" ry="3" />
          <path d="M3 12a9 3 0 0 0 5 2.69M21 9.3V5" />
          <path d="M3 5v14a9 3 0 0 0 6.47 2.88M12 12v4h4" />
          <path d="M13 20a5 5 0 0 0 9-3a4.5 4.5 0 0 0-4.5-4.5c-1.33 0-2.54.54-3.41 1.41L12 16" />
        </g>
      </svg>
    ),
  },
  {
    widthClass: "w-[50rem] lg:w-[60rem] 2xl:w-[100rem]",
    leftIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <path d="M12 5a3 3 0 1 0-5.997.125a4 4 0 0 0-2.526 5.77a4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z" />
          <path d="M9 13a4.5 4.5 0 0 0 3-4M6.003 5.125A3 3 0 0 0 6.401 6.5m-2.924 4.396a4 4 0 0 1 .585-.396M6 18a4 4 0 0 1-1.967-.516M12 13h4m-4 5h6a2 2 0 0 1 2 2v1M12 8h8m-4 0V5a2 2 0 0 1 2-2" />
          <circle cx="16" cy="13" r=".5" />
          <circle cx="18" cy="3" r=".5" />
          <circle cx="20" cy="21" r=".5" />
          <circle cx="20" cy="8" r=".5" />
        </g>
      </svg>
    ),
    rightIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <path d="m15.228 16.852l-.923-.383m.923 2.679l-.923.383M16 2v4m.47 8.305l.382.923m0 5.544l-.383.924m2.679-6.468l.383-.923m-.001 7.391l-.382-.924m1.624-3.92l.924-.383m-.924 2.679l.924.383M21 10.592V6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h6M3 10h18M8 2v4" />
          <circle cx="18" cy="18" r="3" />
        </g>
      </svg>
    ),
  },
  {
    widthClass: "w-[35rem] lg:w-[45rem] 2xl:w-[70rem]",
    leftIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <path d="M2 9.5a5.5 5.5 0 0 1 9.591-3.676a.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5" />
          <path d="M3.22 13H9.5l.5-1l2 4.5l2-7l1.5 3.5h5.27" />
        </g>
      </svg>
    ),
    rightIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <path d="m10.852 14.772l-.383.923m2.679-.923a3 3 0 1 0-2.296-5.544l-.383-.923m2.679.923l.383-.923" />
          <path d="m13.53 15.696l-.382-.924a3 3 0 1 1-2.296-5.544m3.92 1.624l.923-.383m-.923 2.679l.923.383" />
          <path d="M4.5 10H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-.5m-15 4H4a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2h-.5M6 18h.01M6 6h.01m3.218 4.852l-.923-.383m.923 2.679l-.923.383" />
        </g>
      </svg>
    ),
  },
  {
    widthClass: "w-[50rem] lg:w-[55rem] 2xl:w-[90rem]",
    leftIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g
          fill="none"
          stroke="currentColor"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
        >
          <path d="m16 22l-1-4m4-4a1 1 0 0 0 1-1v-1a2 2 0 0 0-2-2h-3a1 1 0 0 1-1-1V4a2 2 0 0 0-4 0v5a1 1 0 0 1-1 1H6a2 2 0 0 0-2 2v1a1 1 0 0 0 1 1" />
          <path d="M19 14H5l-1.973 6.767A1 1 0 0 0 4 22h16a1 1 0 0 0 .973-1.233zM8 22l1-4" />
        </g>
      </svg>
    ),
    rightIcon: (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
      >
        <g fill="none">
          <path d="m12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035q-.016-.005-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427q-.004-.016-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093q.019.005.029-.008l.004-.014l-.034-.614q-.005-.018-.02-.022m-.715.002a.02.02 0 0 0-.027.006l-.006.014l-.034.614q.001.018.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z" />
          <path
            fill="currentColor"
            d="M6.315 6.176c-.25-.638-.24-1.367-.129-2.034a6.8 6.8 0 0 1 2.12 1.07c.28.214.647.283.989.18A9.3 9.3 0 0 1 12 5c.961 0 1.874.14 2.703.391c.342.104.709.034.988-.18a6.8 6.8 0 0 1 2.119-1.07c.111.667.12 1.396-.128 2.033c-.15.384-.075.826.208 1.14C18.614 8.117 19 9.04 19 10c0 2.114-1.97 4.187-5.134 4.818c-.792.158-1.101 1.155-.495 1.726c.389.366.629.882.629 1.456v3a1 1 0 0 0 2 0v-3c0-.57-.12-1.112-.334-1.603C18.683 15.35 21 12.993 21 10c0-1.347-.484-2.585-1.287-3.622c.21-.82.191-1.646.111-2.28c-.071-.568-.17-1.312-.57-1.756c-.595-.659-1.58-.271-2.28-.032a9 9 0 0 0-2.125 1.045A11.4 11.4 0 0 0 12 3c-.994 0-1.953.125-2.851.356a9 9 0 0 0-2.125-1.045c-.7-.24-1.686-.628-2.281.031c-.408.452-.493 1.137-.566 1.719l-.005.038c-.08.635-.098 1.462.112 2.283C3.484 7.418 3 8.654 3 10c0 2.992 2.317 5.35 5.334 6.397A4 4 0 0 0 8 17.98l-.168.034c-.717.099-1.176.01-1.488-.122c-.76-.322-1.152-1.133-1.63-1.753c-.298-.385-.732-.866-1.398-1.088a1 1 0 0 0-.632 1.898c.558.186.944 1.142 1.298 1.566c.373.448.869.916 1.58 1.218c.682.29 1.483.393 2.438.276V21a1 1 0 0 0 2 0v-3c0-.574.24-1.09.629-1.456c.607-.572.297-1.568-.495-1.726C6.969 14.187 5 12.114 5 10c0-.958.385-1.881 1.108-2.684c.283-.314.357-.756.207-1.14"
          />
        </g>
      </svg>
    ),
  },
];

function ServiceBlockComponent(props: {
  left?: boolean;
  children: JSX.Element;
}) {
  return (
    <div className={"flex items-center"}>
      <div
        className={`absolute w-full bg-fd-background h-full z-1000 ${props.left ? "right-full" : "left-full"}`}
      ></div>
      <div
        className={`relative p-0.5 ${props.left ? "bg-linear-to-r" : "bg-linear-to-l"} from-fd-foreground/30 to-fd-background-dark rounded z-100`}
      >
        <div className={"size-8 2xl:size-12 rounded bg-fd-background"} />
        <div
          className={
            "absolute size-full flex items-center justify-center text-xl lg:text-2xl 2xl:text-3xl inset-0 opacity-25"
          }
        >
          {props.children}
        </div>
      </div>
    </div>
  );
}

function ChipWireComponent(props: { left?: boolean }) {
  const ping = React.useRef<HTMLDivElement>(null);
  const outerPingContainer = React.useRef<HTMLDivElement>(null);
  const [usesSecondary, setUsesSecondary] = React.useState<boolean>(false);

  // biome-ignore lint/correctness/useExhaustiveDependencies: onPingAnimUpdate is a static function, nothing else
  useEffect(() => {
    if (!ping.current) return;
    const randLoopDelay = Math.random() * (200 - 100) + 100;
    const randDuration = (Math.random() * (2000 - 1000) + 1000);
    const timeline = createTimeline({
      loop: true,
      delay: 2000,
      duration: randDuration,
      loopDelay: randLoopDelay,
      playbackEase: "linear",
    });

    const moveProp = props.left ? "left" : "right";
    ping.current.style[moveProp] = "101%";

    timeline
      .set(ping.current, { opacity: 1, onBegin: () => setUsesSecondary(false) })
      .add(ping.current, {
        [moveProp]: "-8%",
        scale: {
          from: 1,
          to: 0,
          duration: randDuration * 1.7,
          ease: "easeOutCubic",
        },
        ease: "outQuad",
      })
      .add(ping.current, {
        [moveProp]: "101%",
        scale: {
          from: 1,
          to: 0,
          duration: randDuration * 1.7,
          ease: "easeOutCubic",
        },
        ease: "outQuad",
        onBegin: () => setUsesSecondary(true),
      });
  }, [ping]);

  return (
    <div
      className={`relative ${styles["wire-container"]}`}
      ref={outerPingContainer}
    >
      <div
        className={`absolute w-full h-1 ${props.left ? "bg-linear-to-l" : "bg-linear-to-r"} from-fd-foreground/30 from-10% to-fd-foreground flex items-center`}
      >
        <div
          className={`absolute flex items-center opacity-0 ${props.left ? "left-[101%]" : "right-[101%]"}`}
          ref={ping}
        >
          <div
            className={`rounded-full size-4 bg-white border-2 ${usesSecondary ? "border-fd-brand-secondary" : "border-fd-brand-primary"} z-20`}
          />
          <div
            className={`absolute blur-md saturate-200  rounded-full size-6 ${usesSecondary ? "bg-fd-brand-secondary" : "bg-fd-brand-primary"} z-10`}
          />
          <div
            className={`absolute min-h-2 w-18 2xl:w-36 ${
              usesSecondary
                ? `${props.left ? "bg-linear-to-l right-full" : "bg-linear-to-r left-full"} from-fd-brand-secondary`
                : `${props.left ? "bg-linear-to-r left-full" : "bg-linear-to-l right-full"} from-fd-brand-primary`
            } to-transparent 
          `}
          />
        </div>
      </div>
    </div>
  );
}

function ChipTransmissionComponent(props: {
  widthClass: string;
  leftIcon: JSX.Element;
  rightIcon: JSX.Element;
  idx: number;
}) {
  const wireChipContainer = React.useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!wireChipContainer.current) return;
    animate(wireChipContainer.current, {
      duration: 500,
      delay: 500,
      scale: [0, 1],
    });
  }, []);

  return (
    <div
      className={`w-full relative justify-center items-center ${props.idx >= 3 ? "hidden lg:flex" : "flex"}`}
      ref={wireChipContainer}
    >
      <div
        className={`absolute flex justify-between items-center ${props.widthClass}`}
      >
        <ServiceBlockComponent left={false}>
          {props.rightIcon}
        </ServiceBlockComponent>
        <ServiceBlockComponent left={true}>
          {props.leftIcon}
        </ServiceBlockComponent>
      </div>
      <div
        className={`absolute justify-between items-center flex ${props.widthClass} px-9`}
      >
        <ChipWireComponent />
        <ChipWireComponent left />
      </div>
      <div className={"w-66 lg:w-76 2xl:w-104 flex justify-between items-center z-10"}>
        <div className="size-5 lg:size-6 rounded-full bg-fd-background border-3 border-fd-foreground" />
        <div className="size-5 lg:size-6 rounded-full bg-fd-background border-3 border-fd-foreground" />
      </div>
    </div>
  );
}

export function ChronoGrapherWireComponent() {
  const parentContainer = React.useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!parentContainer.current) return;
    animate(parentContainer.current, {
      duration: 1500,
      y: [20, 0],
      opacity: [0, 1],
    });
  }, []);

  return (
    <div className={`relative opacity-0 -mb-36 lg:-mb-20 2xl:-mb-12 w-full h-full flex flex-col items-center ${styles["wire-container-outer"]}`} ref={parentContainer}>
      <div className="z-100 p-0.5 bg-linear-to-b from-fd-foreground from-30% via-fd-muted via-70% to-fd-background rounded-lg">
        <div
          className={
            "relative size-60 lg:size-68 2xl:size-96 flex items-center justify-center bg-linear-to-b from-fd-background-light " +
            "to-fd-background-dark to-80% rounded-lg **:pointer-events-none **:select-none"
          }
        >
          <div className="w-full h-full *:p-8 *:lg:p-10 *:2xl:p-16 mask-b-from-20% from-white to-transparent">
            <Image
              src={
                useFumadocsTheme() === "light"
                  ? ChronoGrapherDark
                  : ChronoGrapherLight
              }
              alt={"ChronoGrapher Logo"}
              fill
            />
          </div>
          <div className="z-10 absolute opacity-80 w-full h-full mix-blend-overlay *:p-8 *:lg:p-10 *:2xl:p-16 mask-b-from-20% from-white to-transparent">
            <Image
              src={
                useFumadocsTheme() === "light"
                  ? ChronoGrapherDark
                  : ChronoGrapherLight
              }
              alt={"ChronoGrapher Logo"}
              fill
            />
          </div>
          <div className="z-20 absolute blur-lg opacity-60 w-full h-full *:p-8 *:lg:p-10 *:2xl:p-16 mask-b-from-20% from-white to-transparent saturate-200">
            <Image
              src={
                useFumadocsTheme() === "light"
                  ? ChronoGrapherDark
                  : ChronoGrapherLight
              }
              alt={"ChronoGrapher Logo"}
              fill
            />
          </div>
        </div>
      </div>
      <div className="absolute gap-5 flex flex-col items-center justify-center h-full w-full -mt-24 lg:-mt-18 2xl:-mt-8">
        {WIRES.map((x, idx) => (
          <ChipTransmissionComponent
            {...x}
            idx={idx}
            key={
              // biome-ignore lint/suspicious/noArrayIndexKey: Again the array is constant, dumbass
              `transmission-${idx}`
            }
          />
        ))}
      </div>
    </div>
  );
}
