import com.bitwig.flt.document.core.master.DXq;
import java.util.List;
import java.util.function.BiConsumer;
import java.util.function.DoubleConsumer;

/** Native 6.1 export callback seam. Loaded with the host as parent, never a UI scraper. */
public final class ExportTask61 extends Zv {
    private final BiConsumer<Boolean,String> completed;
    private final DoubleConsumer progress;
    public ExportTask61(DXq document,List<?> tracks,String[] names,double start,double end,
                        BiConsumer<Boolean,String> completed,DoubleConsumer progress) {
        super(document,tracks,names,true,start,end);this.completed=completed;this.progress=progress;
    }
    @Override protected boolean jaQ(){return false;}
    @Override protected void v33(boolean success,String error) {
        try {super.v33(success,error);}
        finally {completed.accept(success,error);}
    }
    @Override public void v33(double value){super.v33(value);progress.accept(value);}
}
