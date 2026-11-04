//! RED/GREEN do JSON das propriedades: objeto inteiro, `null` com motivo
//! ou ausente com `--sem-propriedades`; `nao_lidos` diz o que faltou.
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
        tipo: Some("Tipo Sintetico: Teste".into()),
        especificacao: "Especificacao sintetica".into(),
        interessados: vec!["Pessoa Sintetica A".into()],
        assuntos: vec![
            Assunto {
                codigo: Some("000.01".into()),
                descricao: "Assunto sintetico".into(),
            },
            Assunto {
                codigo: None,
                descricao: "Sem codigo".into(),
            },
        ],
        observacoes: String::new(),
        nivel_de_acesso: NivelDeAcesso::Restrito,
        hipotese_legal: Some("Hipotese Sintetica".into()),
        data_de_geracao: None,
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
                {"codigo": null, "descricao": "Sem codigo"}
            ],
            "observacoes": "",
            "nivel_de_acesso": "restrito",
            "hipotese_legal": "Hipotese Sintetica",
            "data_de_geracao": null,
            "fonte": "procedimento_consultar"
        })
    );
    assert_eq!(valor["nao_lidos"], json!(["andamentos", "marcadores"]));
    assert!(valor.get("propriedades_nao_lidas_motivo").is_none());
}

#[test]
fn nao_publicadas_saem_null_com_motivo_e_omitidas_tiram_a_chave() {
    let mut sem_link = aberto(None);
    sem_link.propriedades = Propriedades::NaoPublicadasNaBarra;
    let valor = json_do_processo(&nup(), &sem_link, true);
    assert!(valor.get("propriedades").is_some_and(Value::is_null));
    assert_eq!(
        valor["propriedades_nao_lidas_motivo"],
        "tela_de_propriedades_nao_publicada_na_barra"
    );
    assert_eq!(
        valor["nao_lidos"],
        json!(["andamentos", "marcadores", "propriedades"])
    );
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
        texto.contains("Não lidos nesta versão: marcadores."),
        "{texto}"
    );
    let sem = humano(&nup(), &aberto(None), true);
    assert!(
        sem.contains("Propriedades não lidas (omitidas_por_sem_propriedades)."),
        "{sem}"
    );
    assert!(
        sem.contains("Não lidos nesta versão: marcadores e propriedades."),
        "{sem}"
    );
}
