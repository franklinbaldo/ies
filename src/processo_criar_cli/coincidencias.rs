//! Apresentação de evidências de reconciliação #245, sem interpretar subcódigos.
use crate::processo_criar_fluxo::EvidenciaDeCoincidencia;
use serde_json::json;

pub(super) fn anexar_coincidencias_json(
    valor: &mut serde_json::Value,
    evidencia: EvidenciaDeCoincidencia,
    nups: &[String],
) {
    match evidencia {
        EvidenciaDeCoincidencia::UnicaSemProvaCausal => {
            valor["desfecho"] = json!("possivel_residuo");
            valor["candidatos_compativeis"] = json!(nups);
        }
        EvidenciaDeCoincidencia::Multiplas => {
            valor["desfecho"] = json!("candidatos_ambiguos");
            valor["candidatos_compativeis"] = json!(nups);
        }
        EvidenciaDeCoincidencia::Parcial => {
            valor["coincidencias_parciais_conferidas"] = json!(nups);
        }
    }
}

pub(super) fn informar_coincidencias_humanas(nups: &[String], evidencia: EvidenciaDeCoincidencia) {
    match evidencia {
        EvidenciaDeCoincidencia::UnicaSemProvaCausal => {
            eprintln!(
                "IES: um candidato confere, mas a leitura não demonstra que este POST o criou."
            );
        }
        EvidenciaDeCoincidencia::Multiplas => {
            eprintln!(
                "IES: {} candidatos coincidentes; autoria da criação ambígua.",
                nups.len()
            );
        }
        EvidenciaDeCoincidencia::Parcial => {
            eprintln!(
                "IES: {} coincidências já conferidas; outras leituras falharam, sem prova de aplicação ou unicidade.",
                nups.len()
            );
        }
    }
    for nup in nups {
        eprintln!("  {nup}");
    }
}
