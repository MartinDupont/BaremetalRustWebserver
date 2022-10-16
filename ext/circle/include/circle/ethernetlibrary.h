//
// bcm54213.h
//
// This driver has been ported from the Linux drivers:
//	Broadcom GENET (Gigabit Ethernet) controller driver
//	Broadcom UniMAC MDIO bus controller driver
//	Copyright (c) 2014-2017 Broadcom
//	Licensed under GPLv2
//
// Circle - A C++ bare metal environment for Raspberry Pi
// Copyright (C) 2019-2020  R. Stange <rsta2@o2online.de>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
#ifndef _circle_ethernetlibrary_h
#define _circle_ethernetlibrary_h

#include <circle/macaddress.h>
#include <circle/netdevice.h>
#include <circle/bcm54213.h>
#include <assert.h>


extern "C" {
    boolean USPiInitialize (void);

    const CMACAddress *USPiGetMACAddress (void);

    // returns TRUE if TX ring has currently free buffers
    boolean USPiIsSendFrameAdvisable (void);

    boolean USPiSendFrame (const void *pBuffer, unsigned nLength);

    // pBuffer must have size FRAME_BUFFER_SIZE
    boolean USPiReceiveFrame (void *pBuffer, unsigned *pResultLength);

    // returns TRUE if PHY link is up
    boolean USPiIsLinkUp (void);

    TNetDeviceSpeed USPiGetLinkSpeed (void);

    // update device settings according to PHY status
    boolean USPiUpdatePHY (void);

}


#endif
