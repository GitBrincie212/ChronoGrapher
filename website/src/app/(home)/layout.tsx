import { HomeLayout } from "fumadocs-ui/layouts/home";
import ProgrammingLangToggle from "@/components/ui/toggle-language";
import { baseOptions } from "@/lib/layout.shared";

export default function Layout({ children }: LayoutProps<"/">) {
  return (
    <HomeLayout
      {...baseOptions()}
      links={[
        {
          type: "custom",
          secondary: true,
          children: <ProgrammingLangToggle variant="nav" />,
        },
      ]}
    >
      {children}
    </HomeLayout>
  );
}
