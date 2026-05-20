package com.sora.showcase;

public enum MailType {
    System,
    Event,
    Compensation;

    static MailType decode(SoraReader reader) {
        switch (reader.readU32()) {
            case 0:
                return System;
            case 1:
                return Event;
            case 2:
                return Compensation;
            default:
                throw new SoraReadException("invalid enum ordinal for MailType");
        }
    }
}
