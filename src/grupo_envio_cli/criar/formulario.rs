//! Formulário de criação, identidade vazia e serialização nativa.
use super::Membro;
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
    localizar_formulario, valor_da_lupa,
};
use crate::instalacao::Instalacao;
use crate::tela_html::arvore;

const CRIAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::GrupoUnidadeCadastrar,
    formulario: "frmGrupoCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmCadastrarGrupoUnidade"),
    campos_ocultos_exigidos: &["hdnIdGrupoUnidade", "hdnUnidades"],
    campos_ocultos_reescritos: &["hdnUnidades"],
    campos_usuario: &["txtNome", "txaDescricao"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::GrupoEnvioCriadoNaUnidade,
    },
};

/// Tela já aberta por Novo a partir da lista da unidade conferida.
/// O ID vazio é evidência de cadastro, não um ID a fabricar ou reescrever.
/// Nome/descrição e membros já resolvidos antes da preparação.
pub(super) fn preparar(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    nome: &str,
    descricao: &str,
    membros: &[Membro],
) -> anyhow::Result<SubmissaoDerivada> {
    let dom = arvore(tela.html);
    let form = localizar_formulario(&dom, CRIAR.formulario)?;
    let ids: Vec<_> = form
        .elementos("input")
        .filter(|n| n.atributo("name") == Some("hdnIdGrupoUnidade"))
        .collect();
    anyhow::ensure!(
        matches!(ids.as_slice(), [id] if id.atributo("value") == Some("")),
        "Cadastro não publicou identidade vazia"
    );
    let unidades = valor_da_lupa(membros.iter().map(|m| (m.id.as_str(), m.nome.as_str())));
    Ok(derivar_submissao(
        &CRIAR,
        instalacao,
        tela,
        Some(unidade),
        &[
            ("txtNome", nome),
            ("txaDescricao", descricao),
            ("hdnUnidades", &unidades),
        ],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formulario_web::CodificacaoDaTela;
    use crate::tela_html::HtmlServido;
    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const INTERIOR: &str = "<input type='hidden' name='hdnIdGrupoUnidade' value=''>\
        <input type='text' name='txtNome' value=''>\
        <textarea name='txaDescricao'></textarea>\
        <select name='selUnidades' multiple></select>\
        <input type='hidden' name='hdnUnidades' value=''>\
        <input type='hidden' name='hdnInfraTipoPagina' value='7'>\
        <input type='submit' name='sbmCadastrarGrupoUnidade' value='Salvar'>";
    fn derivar(
        interior: &str,
        codificacao: CodificacaoDaTela,
    ) -> anyhow::Result<SubmissaoDerivada> {
        let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
        let url = format!(
            "https://sei.exemplo.gov.br/sei/controlador.php?acao=grupo_unidade_cadastrar&infra_hash={HASH}"
        );
        let html = HtmlServido::de_teste(&format!(
            "<form id='frmGrupoCadastro' method='post' action='controlador.php?acao=grupo_unidade_cadastrar&infra_hash={HASH}'>{interior}</form>"
        ));
        preparar(
            &instalacao,
            &TelaServida {
                url: &url,
                html: &html,
                codificacao,
            },
            "TESTE",
            "Ação & B",
            "",
            &[Membro {
                id: "001".into(),
                nome: "Órgão".into(),
            }],
        )
    }
    #[test]
    fn corpo_iso_preserva_identidade_vazia_e_ocultos() {
        let post = derivar(INTERIOR, CodificacaoDaTela::Iso88591).unwrap();
        assert_eq!(
            post.corpo(),
            b"hdnIdGrupoUnidade=&txtNome=A%E7%E3o+%26+B&txaDescricao=&hdnUnidades=001%B1%D3rg%E3o&hdnInfraTipoPagina=7&sbmCadastrarGrupoUnidade=Salvar"
        );
    }
    #[test]
    fn corpo_utf8_e_lexical() {
        let post = derivar(INTERIOR, CodificacaoDaTela::Utf8).unwrap();
        assert_eq!(
            post.corpo(),
            b"hdnIdGrupoUnidade=&txtNome=A%C3%A7%C3%A3o+%26+B&txaDescricao=&hdnUnidades=001%C2%B1%C3%93rg%C3%A3o&hdnInfraTipoPagina=7&sbmCadastrarGrupoUnidade=Salvar"
        );
    }
    #[test]
    fn identidade_ausente_preenchida_ou_duplicada_nao_prepara() {
        for interior in [
            INTERIOR.replace("name='hdnIdGrupoUnidade'", "name='outro'"),
            INTERIOR.replace(
                "name='hdnIdGrupoUnidade' value=''",
                "name='hdnIdGrupoUnidade' value='001'",
            ),
            INTERIOR.replace(
                "name='hdnIdGrupoUnidade' value=''",
                "name='hdnIdGrupoUnidade'",
            ),
            format!("{INTERIOR}<input type='hidden' name='hdnIdGrupoUnidade' value=''>"),
        ] {
            assert!(derivar(&interior, CodificacaoDaTela::Iso88591).is_err());
        }
    }
    #[test]
    fn exige_botao_e_oculto_da_lupa() {
        for interior in [
            INTERIOR.replace("sbmCadastrarGrupoUnidade", "sbmAlterarGrupoUnidade"),
            INTERIOR.replace("name='hdnUnidades'", "name='outro'"),
        ] {
            assert!(derivar(&interior, CodificacaoDaTela::Iso88591).is_err());
        }
    }
}
