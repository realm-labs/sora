package external;

import com.sora.showcase.Item;
import com.sora.showcase.ItemTable;
import com.sora.showcase.LocalePack;
import com.sora.showcase.SoraConfig;
import com.sora.showcase.SoraI18n;

final class PublicApiSmoke {
    static int loadAndRead(byte[] configBytes, byte[] localeBytes) {
        SoraConfig config = SoraConfig.fromBytes(configBytes);
        ItemTable items = config.item();
        SoraConfig.TableInfo info = items.info();
        Item item = items.get(1001);

        SoraI18n i18n = new SoraI18n();
        i18n.mount(config, LocalePack.parse(localeBytes));

        return item == null ? info.indexes().size() : item.id();
    }
}
