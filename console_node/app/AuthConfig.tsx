import { NextAuthOptions, JWT, User, Account, Profile } from "next-auth";
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
    session: async ({ session, token }) => {
      console.log(session);
      return session;
    },
    jwt: async ({ token }) => {
      return token;
    },
  },
  session: {
    strategy: "jwt",
  },
};
