"use client";

import type React from "react";
import Highlight from "src/components/ui/highlight";
import styles from "./ScalabilityShowcaseSection.module.css";
import SectionWire from "@/components/home/SectionWire";

function FeatureShowcaseBlock(props: {
  title: string;
  children: React.ReactElement;
}) {
  return (
    <div
      className={
        "w-96 h-42 rounded-lg ring-2 ring-fd-muted-foreground/40 overflow-hidden " +
        "ring-offset-2 ring-offset-fd-background bg-fd-muted-foreground/10 " +
        "backdrop-blur-sm dark:backdrop-blur-md relative"
      }
    >
      <div
        className={
          "flex items-center justify-center object-cover p-10 w-full h-full"
        }
      >
        {props.children}
      </div>
    </div>
  );
}

export default function ScalabilityShowcaseSection() {
  return (
    <SectionWire
      leftRight={true}
      height={28}
      icon={
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
          <path d="M0 0h24v24H0z" fill="none" />
          <g fill="none" stroke="currentColor" strokeLinecap="round" strokeWidth="1.5">
            <path d="M2 12c0 4.714 0 7.07 1.465 8.535c1.464 1.464 3.821 1.464 8.535 1.464s7.071 0 8.536-1.464c1.357-1.357 1.456-3.48 1.464-7.536M11 2c-4.055.007-6.178.107-7.535 1.464C2.648 4.28 2.287 5.374 2.127 7" />
            <path strokeLinejoin="round" d="m13 11l9-9m0 0h-5.344M22 2v5.344M21 3l-9 9m0 0h4m-4 0V8" />
          </g>
        </svg>

      }
      title={
        <>
          Built For Scalability <u>From Day One</u>
        </>
      }
      description={
        <>
          <span className={"opacity-60"}>
            ChronoGrapher is designed to scale from your home laptop to a
            distributed service without switching to a different framework.
          </span>
          <span> </span>
          <Highlight color={"info"}>
            Pay for what your infrastructure is in need of.
          </Highlight>
        </>
      }
      iconColorClass={"bg-fd-info/10 dark:bg-fd-info/20 border-fd-info text-fd-info"}
    >
      <div
        className={
          "-ml-5 border-t-2 h-full border-dashed relative border-fd-muted-foreground/20"
        }
      >
        <div className={styles["booster-bg"]} />
        <div className={styles["booster-bg-blur-mask"]} />
        <div className={"pl-6 z-10 flex flex-1 items-center gap-4 h-full"}>
          <FeatureShowcaseBlock title={"Run Locally"}>
            <div></div>
          </FeatureShowcaseBlock>
          <FeatureShowcaseBlock title={"Add Persistence"}>
            <div></div>
          </FeatureShowcaseBlock>
          <FeatureShowcaseBlock title={"Scale To The Cloud"}>
            <div></div>
          </FeatureShowcaseBlock>
          <FeatureShowcaseBlock title={"Go Full-Distributed"}>
            <div></div>
          </FeatureShowcaseBlock>
        </div>
      </div>
    </SectionWire>
  );
}
