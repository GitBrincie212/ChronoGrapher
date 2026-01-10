import { waapi } from "animejs";
import React, { useEffect } from "react";
import CallToActionButton from "@/components/home/call-to-action-button";
import { ChronoGrapherWireComponent } from "@/components/home/chronographer-wire-component";
import Highlight from "@/components/ui/highlight";

import animate = waapi.animate;

function ChronoGrapherCallToActionText() {
  const titleText = React.useRef<HTMLHeadingElement>(null);
  const paragraphText = React.useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!titleText.current || !paragraphText.current) return;
    animate(titleText.current, {
      duration: 1000,
      opacity: [0, 1],
      y: [-20, 0],
    });

    animate(paragraphText.current, {
      duration: 2000,
      opacity: [0, 1],
      y: [-10, 0],
    });
  }, []);

  return (
    <div className={"contents *:opacity-0"}>
      <h1
        className={"text-center w-screen font-semibold text-4xl z-100"}
        ref={titleText}
      >
        One Unified{" "}
        <Highlight pulse={true} color={"primary"}>
          Scheduler
        </Highlight>
        , Unlimited{" "}
        <Highlight pulse={true} color={"secondary"}>
          Power
        </Highlight>
      </h1>
      <div
        className="text-center w-screen font-light opacity-35"
        ref={paragraphText}
      >
        One composable, scalable engine instead of many brittle schedulers.
      </div>
    </div>
  );
}

export default function HeroSection() {
  return (
    <div className="w-screen h-96 mt-16 mb-48 relative">
      <ChronoGrapherWireComponent />
      <ChronoGrapherCallToActionText />
      <div className="flex gap-4 w-screen justify-center items-center mt-8">
        <CallToActionButton
          title={"Getting Started"}
          variant={"primary"}
          href={"/docs/installation"}
        />
        <CallToActionButton
          title={"View Roadmap"}
          variant={"secondary"}
          href={""}
        />
      </div>
    </div>
  );
}
