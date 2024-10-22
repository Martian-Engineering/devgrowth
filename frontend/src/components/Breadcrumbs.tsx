import * as React from "react";
import { useState } from "react";
import { ChevronDownIcon } from "@radix-ui/react-icons";
import { Slash } from "lucide-react";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useProfile } from "@/contexts/ProfileContext";

interface BreadcrumbItem {
  href: string;
  label: string;
  dropdown?: Array<{ href: string; label: string }>;
}

export default function Breadcrumbs() {
  const pathname = usePathname();
  const { profile } = useProfile();

  const breadcrumbs: BreadcrumbItem[] = React.useMemo(() => {
    const parts = pathname.split("/").filter(Boolean);
    const items: BreadcrumbItem[] = [{ href: "/", label: "Home" }];

    if (parts[0] === "collections") {
      items.push({
        href: "/collections",
        label: "Collections",
        dropdown:
          profile?.collections?.map((c) => ({
            href: `/collections/${c.collection_id}`,
            label: c.name,
          })) || [],
      });

      if (parts[1]) {
        const collectionId = parseInt(parts[1], 10);
        const collection = profile?.collections?.find(
          (c) => c.collection_id === collectionId,
        );

        if (collection) {
          items.push({
            href: `/collections/${collectionId}`,
            label: collection.name,
          });
        }
      }
    }

    return items;
  }, [pathname, profile]);

  return (
    <Breadcrumb>
      <BreadcrumbList>
        {breadcrumbs.map((crumb, index) => (
          <React.Fragment key={crumb.href}>
            {index > 0 && (
              <BreadcrumbSeparator>
                <Slash />
              </BreadcrumbSeparator>
            )}
            <BreadcrumbItem>
              {crumb.dropdown ? (
                <DropdownMenu>
                  <DropdownMenuTrigger className="flex items-center gap-1">
                    {crumb.label}
                    <ChevronDownIcon />
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    {crumb.dropdown.map((item) => (
                      <DropdownMenuItem key={item.href}>
                        <Link href={item.href}>{item.label}</Link>
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              ) : index === breadcrumbs.length - 1 ? (
                <BreadcrumbPage>{crumb.label}</BreadcrumbPage>
              ) : (
                <BreadcrumbLink asChild>
                  <Link href={crumb.href}>{crumb.label}</Link>
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
          </React.Fragment>
        ))}
      </BreadcrumbList>
    </Breadcrumb>
  );
}
