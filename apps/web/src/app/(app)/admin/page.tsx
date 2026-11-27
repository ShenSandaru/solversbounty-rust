import type { Metadata } from "next";
import { AdminLoginPage } from "@/features/admin/admin-login-page";

export const metadata: Metadata = {
  title: "Admin sign-in",
};

export default function Page() {
  return <AdminLoginPage />;
}
