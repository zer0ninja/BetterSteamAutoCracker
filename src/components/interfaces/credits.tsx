import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";

export function CreditsInterface() {
  return (
    <div className="flex-1 p-6 space-y-6 flex flex-col items-center">
      <div className="text-center">
        <h1 className="text-3xl font-bold tracking-tight text-foreground">
          Credits
        </h1>
        <p className="text-lg text-muted-foreground leading-relaxed">
          Contributors who made this project possible
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-4xl w-full">
        <Card className="border-border bg-card shadow-2xl">
          <CardHeader>
            <div className="flex items-center gap-4">
              <Avatar className="w-14 h-14">
                <AvatarImage
                  src="/avatars/detanup.jpg"
                  alt="Detanup01 & Mr.Goldberg"
                />
                <AvatarFallback>GD</AvatarFallback>
              </Avatar>
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <CardTitle className="text-xl font-semibold text-foreground">
                    Detanup01 & Mr.Goldberg
                  </CardTitle>
                </div>
                <Badge variant="secondary" className="text-xs">
                  Steam Emulator
                </Badge>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">
              Developed the Goldberg fork for Steam API emulation
            </p>
          </CardContent>
        </Card>

        <Card className="border-border bg-card shadow-2xl">
          <CardHeader>
            <div className="flex items-center gap-4">
              <Avatar className="w-14 h-14">
                <AvatarImage src="/avatars/atom0s.jpg" alt="Atom0s" />
                <AvatarFallback>AT</AvatarFallback>
              </Avatar>
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <CardTitle className="text-xl font-semibold text-foreground">
                    atom0s
                  </CardTitle>
                </div>
                <Badge variant="secondary" className="text-xs">
                  DRM Removal
                </Badge>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">
              Created Steamless for removing Steam DRM protection
            </p>
          </CardContent>
        </Card>

        <Card className="border-border bg-card shadow-2xl">
          <CardHeader>
            <div className="flex items-center gap-4">
              <Avatar className="w-14 h-14">
                <AvatarImage src="/avatars/Sovereign.jpg" alt="Sovereign" />
                <AvatarFallback>SV</AvatarFallback>
              </Avatar>
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <CardTitle className="text-xl font-semibold text-foreground">
                    Sovereign
                  </CardTitle>
                </div>
                <Badge variant="secondary" className="text-xs">
                  BSAC Developer
                </Badge>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">
              Designed and built the Better Steam AutoCracker application
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
