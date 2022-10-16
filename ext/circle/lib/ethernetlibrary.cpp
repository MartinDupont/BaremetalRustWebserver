//
// ethernetlibrary.cpp
//
// Circle - A C++ bare metal environment for Raspberry Pi
// Copyright (C) 2014-2020  R. Stange <rsta2@o2online.de>
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
#include <circle/ethernetlibrary.h>
#include <circle/netdevice.h>
#include <circle/macaddress.h>
#include <circle/bcm54213.h>
#include <assert.h>

static CNetDevice *my_device = 0;


boolean USPiInitialize (void)
{
	assert (my_device == 0);

	boolean bOK = TRUE;

	CBcm54213Device m_Bcm54213;

    bOK = m_Bcm54213.Initialize ();

    if (bOK) {
        my_device = CNetDevice::GetNetDevice (0);
    }

	return bOK;
}

// TODO: Implement GetMacAddress!!!!
const CMACAddress *USPiGetMACAddress (void)
{
    return my_device->GetMACAddress();
}

// returns TRUE if TX ring has currently free buffers
boolean USPiIsSendFrameAdvisable (void)
{
    return my_device->IsSendFrameAdvisable ();
}

boolean USPiSendFrame (const void *pBuffer, unsigned nLength)
{
    return my_device->SendFrame (pBuffer, nLength);
}

// pBuffer must have size FRAME_BUFFER_SIZE
boolean USPiReceiveFrame (void *pBuffer, unsigned *pResultLength)
{
    return my_device->ReceiveFrame (pBuffer, pResultLength);
}

// returns TRUE if PHY link is up
boolean USPiIsLinkUp (void)
{
    return my_device->IsLinkUp ();
}

// TODO: Unrwap CMACAddress into primitive type?
TNetDeviceSpeed USPiGetLinkSpeed (void)
{
    return my_device->GetLinkSpeed();
}

// update device settings according to PHY status
boolean USPiUpdatePHY (void)
{
    return my_device->UpdatePHY ();
}
