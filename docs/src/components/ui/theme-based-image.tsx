'use client'

import Image from "next/image";
import {useFumadocsTheme} from "@/components/theme";

export default function ThemeBasedImage(props: {
    dark: string,
    light: string,
    width: number,
    height: number,
    alt?: string
}) {
    return (
        <Image
            src={useFumadocsTheme() === 'dark' ? props.dark : props.light} alt={props.alt ?? ""}
            width={props.width}
            height={props.height}
        />
    );
}