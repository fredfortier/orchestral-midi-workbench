package com.midiworkbench.notes.internal;

import java.lang.classfile.ClassFile;
import java.lang.classfile.ClassHierarchyResolver;
import java.lang.classfile.CodeTransform;
import java.lang.classfile.MethodModel;
import java.lang.classfile.MethodTransform;
import java.lang.classfile.instruction.InvokeInstruction;
import java.lang.constant.ClassDesc;
import java.lang.constant.MethodTypeDesc;
import java.util.Set;
import static java.lang.constant.ConstantDescs.CD_String;
import static java.lang.constant.ConstantDescs.CD_void;

/** Adds checkpoints to a private development copy of the real adapter class. */
public final class NativeFaultTransformer {
    private static final String ADAPTER="com/midiworkbench/notes/internal/EditorAdapter61";
    private static final String AUTOMATION="com/midiworkbench/notes/internal/AutomationAdapter61";
    private static final Set<String> FAMILIES=Set.of("set","remove","moveTime","orderPoints","applyField","placeGeometry","createNote");
    private NativeFaultTransformer() {}
    private static boolean nativeInvoke(java.lang.classfile.CodeElement element){
        return element instanceof InvokeInstruction call && call.owner().asInternalName().equals(ADAPTER) && call.name().stringValue().equals("invoke");
    }
    public static byte[] instrument(byte[] bytes){return instrument(bytes,NativeFaultTransformer.class.getClassLoader());}
    public static byte[] instrument(byte[] bytes,ClassLoader loader){
        ClassFile files=ClassFile.of(ClassFile.ClassHierarchyResolverOption.of(ClassHierarchyResolver.ofResourceParsing(loader)));var model=files.parse(bytes);
        if(model.thisClass().asInternalName().equals(AUTOMATION))return automation(bytes,loader);
        if(!model.thisClass().asInternalName().equals(ADAPTER))throw new IllegalArgumentException("Only the known adapter may be instrumented");
        int[] sites={0};
        byte[] output=files.transformClass(model,(builder,element)->{
            if(element instanceof MethodModel method && FAMILIES.contains(method.methodName().stringValue()) && method.code().isPresent()){
                String family=method.methodName().stringValue();
                long invokes=method.code().orElseThrow().elementStream().filter(NativeFaultTransformer::nativeInvoke).count();
                if(family.equals("set") && invokes!=2)throw new IllegalStateException("Native setter layout changed; inspect the development transformer");
                int[] ordinal={0};
                CodeTransform transform=(code,item)->{
                    code.with(item);
                    if(nativeInvoke(item)){
                        ordinal[0]++;
                        // set() first resolves its descriptor; its second invocation writes.
                        if(!family.equals("set") || ordinal[0]==2){
                            sites[0]++;
                            if(family.equals("set"))code.aload(1);else code.ldc(family+":"+ordinal[0]);
                            code.invokestatic(ClassDesc.of("com.midiworkbench.notes.internal.NativeFaults"),"checkpoint",MethodTypeDesc.of(CD_void,CD_String));
                        }
                    }
                };
                builder.transformMethod(method,MethodTransform.transformingCode(transform));
            }else builder.with(element);
        });
        if(sites[0]<5)throw new IllegalStateException("Expected native mutation checkpoint sites were not found");
        return output;
    }
    private static byte[] automation(byte[] bytes,ClassLoader loader) {
        ClassFile files=ClassFile.of(ClassFile.ClassHierarchyResolverOption.of(ClassHierarchyResolver.ofResourceParsing(loader)));
        int[] sites={0};
        byte[] result=files.transformClass(files.parse(bytes),(builder,element)->{
            if(element instanceof MethodModel method && Set.of("write","set").contains(method.methodName().stringValue())) {
                String family=method.methodName().stringValue();int[] ordinal={0};
                builder.transformMethod(method,MethodTransform.transformingCode((code,item)->{
                    code.with(item);
                    if(item instanceof InvokeInstruction call && ((family.equals("set") && call.owner().asInternalName().equals("com/midiworkbench/notes/internal/Reflection") && call.name().stringValue().equals("call")) || (family.equals("write") && call.owner().asInternalName().equals(AUTOMATION) && call.name().stringValue().equals("call")))) {
                        ordinal[0]++;
                        if(!family.equals("set") || ordinal[0]==2) {
                            sites[0]++;code.ldc("automation:"+family+":"+ordinal[0]);
                            code.invokestatic(ClassDesc.of("com.midiworkbench.notes.internal.NativeFaults"),"checkpoint",MethodTypeDesc.of(CD_void,CD_String));
                        }
                    }
                }));
            }else builder.with(element);
        });
        if(sites[0]<3)throw new IllegalStateException("Expected automation mutation checkpoints missing");
        return result;
    }

}
