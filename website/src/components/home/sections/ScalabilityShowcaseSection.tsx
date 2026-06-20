"use client"

import type React from "react";
import Highlight from "src/components/ui/highlight"
import styles from "./FeatureShowcaseSection.module.css"
import Laptop from "@/assets/Laptop LineArt.png"
import HDD from "@/assets/HDD LineArt.png"
import Image from "next/image";

function FeatureShowcaseBlock(props: {
    title: string;
    description: string;
    children: React.ReactElement
}) {
    return (
        <div className={
            "w-96 h-42 rounded-lg ring-2 ring-fd-muted-foreground/40 overflow-hidden " +
            "ring-offset-2 ring-offset-fd-background bg-fd-muted-foreground/10 " +
            "backdrop-blur-sm dark:backdrop-blur-md relative"
        }>
            <div className={"flex items-center justify-center object-cover p-10 w-full h-full"}>
                {props.children}
            </div>
        </div>
    );
}

export default function ScalabilityShowcaseSection() {
    return (
        <div className={"w-full h-120"}>
            <div className={"ml-16 absolute z-10 size-8 bg-fd-background border border-fd-muted-foreground/20 rounded-lg"}></div>
            <div className={"ml-[4.9rem] border-l-2 border-double border-fd-muted-foreground/20 px-6 py-10 h-full"}>
                <h1 className={"text-xl font-bold"}>Built For Scalability <u>From Day One</u></h1>
                <div className={"w-[70ch] mb-10"}>
                    <span className={"opacity-60"}>
                        ChronoGrapher is designed to scale from your home laptop to a distributed service
                        without switching to a different framework.
                    </span>
                    <span> </span>
                    <Highlight color={"info"}>Pay for what your infrastructure is in need of.</Highlight>
                </div>
                <div className={"-ml-6 border-t-2 border-b-2 h-56 border-dashed relative border-fd-muted-foreground/20"}>
                    <div className={styles["booster-bg"]} />
                    <div className={styles["booster-bg-blur-mask"]} />
                    <div className={"py-4 pl-4 z-10 flex flex-1 items-center gap-4 h-full"}>
                        <FeatureShowcaseBlock title={"Run Locally"}>
                            <Image src={Laptop} alt={""} unoptimized={true} className={"p-12"} />
                        </FeatureShowcaseBlock>
                        <FeatureShowcaseBlock title={"Add Persistence"}>
                            <Image src={HDD} alt={""} unoptimized={true} className={"p-14 pt-18"} />
                        </FeatureShowcaseBlock>
                        <FeatureShowcaseBlock title={"Scale To The Cloud"}>
                            <div></div>
                        </FeatureShowcaseBlock>
                        <FeatureShowcaseBlock title={"Go Full-Distributed"}>
                            <div></div>
                        </FeatureShowcaseBlock>
                    </div>
                </div>
            </div>
        </div>
    );
}