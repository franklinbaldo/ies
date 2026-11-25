//! JSON das propriedades: objeto inteiro (sem `null`) ou ausente com
//! `--sem-propriedades`; `nao_lidos` diz o que foi omitido por pedido.
use super::tests::aberto;
use super::*;
use crate::processo_obter::Propriedades;
use crate::propriedades_processo::{
    Assunto, FonteDasPropriedades, NivelDeAcesso, PropriedadesLidas,
};

fn nup() -> NumeroProcesso {
    NumeroProcesso::novo("00001.000001/2026-01").unwrap()
}

fn com_propriedades() -> ProcessoAberto {
    let mut aberto = aberto(Some("UNIDADE TESTE"));
    aberto.propriedades = Propriedades::Lidas(PropriedadesLidas {
        tipo: "Tipo Sintetico: Teste".into(),
        especificacao: "Especificacao sintetica".into(),
        interessados: vec!["Pessoa Sintetica A".into()],
        assuntos: vec![
            Assunto {
                codigo: "000.01".into(),
                descricao: "Assunto sintetico".into(),
            },
            Assunto {
                codigo: "000.02".into(),
                descricao: "Outro assunto".into(),
            },
        ],
        observacoes: String::new(),
        nivel_de_acesso: NivelDeAcesso::Restrito,
        hipotese_legal: Some("Hipotese Sintetica".into()),
        data_de_geracao: "01/02/2026".into(),
        fonte: FonteDasPropriedades::Consultar,
    });
    aberto
}

#[test]
fn propriedades_lidas_saem_inteiras_e_deixam_nao_lidos() {
    let valor = json_do_processo(&nup(), &com_propriedades(), true);
    assert_eq!(
        valor["propriedades"],
        json!({
            "tipo": "Tipo Sintetico: Teste",
            "especificacao": "Especificacao sintetica",
            "interessados": ["Pessoa Sintetica A"],
            "assuntos": [
                {"codigo": "000.01", "descricao": "Assunto sintetico"},
                {"codigo": "000.02", "descricao": "Outro assunto"}
            ],
            "observacoes": "",
            "nivel_de_acesso": "restrito",
            "hipotese_legal": "Hipotese Sintetica",
            "data_de_geracao": "01/02/2026",
            "fonte": "procedimento_consultar"
        })
    );
    assert_eq!(valor["nao_lidos"], json!(["andamentos", "marcadores"]));
    assert!(valor.get("propriedades_nao_lidas_motivo").is_none());
}

#[test]
fn omitidas_tiram_a_chave_e_so_dizem_o_motivo_pedido() {
    let omitidas = json_do_processo(&nup(), &aberto(None), true);
    assert!(omitidas.get("propriedades").is_none());
    assert_eq!(
        omitidas["propriedades_nao_lidas_motivo"],
        "omitidas_por_sem_propriedades"
    );
}

#[test]
fn texto_humano_diz_as_propriedades_e_o_que_nao_foi_lido() {
    let texto = humano(&nup(), &com_propriedades(), true);
    assert!(
        texto.contains("Propriedades (procedimento_consultar)"),
        "{texto}"
    );
    assert!(texto.contains("nível restrito"), "{texto}");
    assert!(texto.contains("000.01 - Assunto sintetico"), "{texto}");
    assert!(
        texto.contains("Marcadores não lidos (omitidos_por_sem_marcadores)."),
        "{texto}"
    );
    assert!(!texto.contains("Não lidos nesta versão"), "{texto}");
    let sem = humano(&nup(), &aberto(None), true);
    assert!(
        sem.contains("Propriedades não lidas (omitidas_por_sem_propriedades)."),
        "{sem}"
    );
    assert!(!sem.contains("Não lidos nesta versão"), "{sem}");
}
