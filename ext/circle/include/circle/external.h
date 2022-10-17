extern "C" {
    void MsDelay (unsigned nMilliSeconds);
    void usDelay (unsigned nMicroSeconds);
    unsigned GetMicrosecondTicks (void);
    //
    // Interrupt handling
    //
    typedef void TIRQHandler (void *pParam);

    // USPi uses USB IRQ 9
    void ConnectInterrupt (unsigned nIRQ, TIRQHandler *pHandler, void *pParam);

}
