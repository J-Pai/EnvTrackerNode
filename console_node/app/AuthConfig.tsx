import { NextAuthOptions } from "next-auth";
import GoogleProvider from "next-auth/providers/google";

export const authOptions: NextAuthOptions = {
  providers: [
    GoogleProvider({
      clientId: process.env.GOOGLE_OAUTH2_CLIENT_ID as string,
      clientSecret: process.env.GOOGLE_OAUTH2_CLIENT_SECRET as string,
    }),
  ],
  secret: process.env.NEXTAUTH_SECRET,
  callbacks: {
    signIn: async ({ user }) => {
      return true;
    },
    session: async ({ session, token }) => {
      return session;
    },
    redirect: async ({ url, baseUrl }) => {
      return baseUrl;
    },
  },
  pages: {
    signIn: "/404",
    signOut: "/404",
    error: "/404", // Error code passed in query string as ?error=
    verifyRequest: "/404", // (used for check email message)
    newUser: "/404", // New users will be directed here on first sign in (leave the property out if not of interest)
  },
};
