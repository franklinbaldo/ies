//! RED/GREEN do alvo e do plano de `--definir` (contracts/cli/unidade-definir-colunas.md),
//! com telas sintéticas modeladas na estrutura observada da tela
//! `procedimento_configurar_detalhe` (PEN1 5.0.4 e GO 5.0.3: 12 opções;
//! `UnB` 4.0.9: 11, sem Prioridade). Ids, hashes e unidade fictícios.
use super::*;
use crate::erro_html::ErroHtml;
use ColunaDoDetalhe as C;

pub(crate) const HASH_DO_SALVAR: &str =
    "9999999999999999999999999999999999999999999999999999999999999999";

/// Tela servida com o formulário que o POST de salvar repete: `action`
/// relativa assinada, `hdnInfraTipoPagina`, `sbmSalvar`, as caixas
/// publicadas (sem `value`) e `hdnInfraPrefixoCookie` fora do formulário.
pub(crate) fn tela_salvavel(publicadas: &[C], marcadas: &[C], extra: &str) -> String {
    use std::fmt::Write as _;
    let mut opcoes = String::new();
    for &coluna in publicadas {
        let campo = coluna.campo();
        let checked = if marcadas.contains(&coluna) {
            " checked=\"checked\""
        } else {
            ""
        };
        write!(
            opcoes,
            "<div class=\"infraDivCheckbox\"><input type=\"checkbox\" id=\"{campo}\" name=\"{campo}\" \
             class=\"infraCheckbox\" onchange=\"validarNumeroCriterios(this)\"{checked} tabindex=\"501\" />\
             <label for=\"{campo}\" class=\"infraLabelCheckbox\">{}</label></div>",
            coluna.rotulo()
        )
        .unwrap();
    }
    format!(
        "<html><head><title>SEI - Configurar Nível de Detalhe</title></head><body>\
         <form id=\"frmAcessoExternoCancelar\" method=\"post\" onsubmit=\"return OnSubmitForm();\" \
         action=\"controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_configurar_detalhe&amp;infra_sistema=100&amp;infra_unidade_atual=200&amp;infra_hash={HASH_DO_SALVAR}\">\
         <div class=\"infraBarraComandos\"><button type=\"submit\" accesskey=\"S\" id=\"sbmSalvar\" \
         name=\"sbmSalvar\" value=\"Salvar\" class=\"infraButton\"><span class=\"infraTeclaAtalho\">S</span>alvar</button></div>\
         {opcoes}{extra}\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\" />\
         </form><input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"SEI_SINTETICO\" />\
         </body></html>"
    )
}

pub(crate) fn onze() -> Vec<C> {
    C::TODAS
        .into_iter()
        .filter(|c| *c != C::Prioridade)
        .collect()
}

fn nivel(publicadas: &[C], marcadas: &[C]) -> NivelDeDetalhe {
    ler_nivel_de_detalhe(&tela_salvavel(publicadas, marcadas, "")).unwrap()
}

#[test]
fn definir_todas_ou_lista_de_chaves_estaveis() {
    assert_eq!(
        AlvoDasColunas::interpretar("todas"),
        Ok(AlvoDasColunas::Todas)
    );
    assert_eq!(
        AlvoDasColunas::interpretar("tipo, atribuicao ,interessados"),
        Ok(AlvoDasColunas::Lista(vec![
            C::Atribuicao,
            C::TipoProcesso,
            C::Interessados
        ]))
    );
    let todas = C::TODAS.map(C::chave).join(",");
    assert_eq!(
        AlvoDasColunas::interpretar(&todas),
        Ok(AlvoDasColunas::Lista(C::TODAS.to_vec()))
    );
}

#[test]
fn definir_recusa_lista_vazia_item_vazio_chave_desconhecida_e_repetida() {
    use ErroDaListaDeColunas as E;
    for vazio in ["", "  ", ","] {
        assert_eq!(
            AlvoDasColunas::interpretar(vazio),
            Err(if vazio == "," {
                E::ItemVazioNaListaDeColunas
            } else {
                E::ListaDeColunasVazia
            }),
            "{vazio:?}"
        );
    }
    assert_eq!(
        AlvoDasColunas::interpretar("tipo,,anotacao"),
        Err(E::ItemVazioNaListaDeColunas)
    );
    for (texto, chave) in [
        ("Tipo", "Tipo"),
        ("tipo,todas", "todas"),
        ("chkSinAtribuicao", "chkSinAtribuicao"),
        ("processo", "processo"),
    ] {
        assert_eq!(
            AlvoDasColunas::interpretar(texto),
            Err(E::ColunaDesconhecida {
                chave: chave.to_owned()
            }),
            "{texto}"
        );
    }
    assert_eq!(
        AlvoDasColunas::interpretar("tipo,anotacao,tipo"),
        Err(E::ColunaRepetidaNaLista {
            coluna: C::TipoProcesso
        })
    );
}

#[test]
fn todas_sao_as_publicadas_nesta_instalacao() {
    let atual = [C::Atribuicao, C::TipoProcesso, C::Interessados];
    let doze = planejar(&nivel(&C::TODAS, &atual), &AlvoDasColunas::Todas).unwrap();
    assert_eq!(doze.antes, atual);
    assert_eq!(doze.alvo, C::TODAS);
    assert_eq!(doze.indisponiveis, []);
    assert_eq!(doze.marcar().len(), 9);
    assert_eq!(doze.desmarcar(), []);
    assert!(!doze.inalterado());

    let unb = planejar(&nivel(&onze(), &atual), &AlvoDasColunas::Todas).unwrap();
    assert_eq!(unb.alvo, onze());
    assert_eq!(unb.indisponiveis, [C::Prioridade]);
    assert!(!unb.marcar().contains(&C::Prioridade));
}

#[test]
fn lista_marca_e_desmarca_e_alvo_igual_e_inalterado() {
    let atual = [C::Atribuicao, C::TipoProcesso, C::Interessados];
    let lido = nivel(&C::TODAS, &atual);
    let plano = planejar(
        &lido,
        &AlvoDasColunas::Lista(vec![C::Marcadores, C::Atribuicao]),
    )
    .unwrap();
    assert_eq!(plano.alvo, [C::Atribuicao, C::Marcadores]);
    assert_eq!(plano.marcar(), [C::Marcadores]);
    assert_eq!(plano.desmarcar(), [C::TipoProcesso, C::Interessados]);
    let igual = planejar(&lido, &AlvoDasColunas::Lista(atual.to_vec())).unwrap();
    assert!(igual.inalterado());
    assert!(igual.marcar().is_empty() && igual.desmarcar().is_empty());
}

#[test]
fn coluna_nao_publicada_e_recusada_no_plano() {
    let lido = nivel(&onze(), &[]);
    assert_eq!(
        planejar(
            &lido,
            &AlvoDasColunas::Lista(vec![C::Atribuicao, C::Prioridade])
        ),
        Err(ColunaIndisponivelNestaInstalacao {
            coluna: C::Prioridade
        })
    );
}

#[test]
fn salvar_declara_os_campos_do_catalogo_e_o_efeito_persistente() {
    use crate::formulario_web::{
        ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, PreferenciaDoUsuario,
        ResultadoDoEnvio,
    };
    let op = &SALVAR_NIVEL_DE_DETALHE;
    assert_eq!(op.campos_usuario, C::TODAS.map(C::campo));
    assert_eq!(op.envio, EnvioDoFormulario::BotaoDeEnvio("sbmSalvar"));
    assert_eq!(op.campos_ocultos_exigidos, ["hdnInfraTipoPagina"]);
    assert_eq!(op.campos_ocultos_reescritos.len(), 0);
    assert_eq!(op.resultado, ResultadoDoEnvio::ConferidoPorLeituraPosterior);
    assert_eq!(op.unidade, ConferenciaDaUnidade::NaTelaDeOrigemDoLink);
    assert_eq!(
        op.efeito,
        EfeitoAvaliado::AlteraPreferenciaDoUsuario {
            o_que: PreferenciaDoUsuario::ColunasDaVisaoDetalhada
        }
    );
    let plano = planejar(&nivel(&C::TODAS, &[]), &AlvoDasColunas::Todas).unwrap();
    let campos = campos_do_envio(&plano);
    assert_eq!(campos.len(), 12);
    assert!(campos.iter().all(|(_, valor)| *valor == "on"));
    assert_eq!(campos[0], ("chkSinAtribuicao", "on"));
}

#[test]
fn formulario_observado_passa_na_conferencia_de_controles() {
    assert_eq!(
        conferir_controles_do_formulario(&tela_salvavel(&C::TODAS, &[], "")),
        Ok(())
    );
    // Não enviados pelo navegador: botão `button`, `reset` e `disabled`.
    let inertes = "<button type=\"button\" name=\"btnFechar\" value=\"Fechar\">Fechar</button>\
                   <input type=\"reset\" name=\"btnLimpar\" value=\"Limpar\">\
                   <input type=\"text\" name=\"txtDesligado\" disabled>";
    assert_eq!(
        conferir_controles_do_formulario(&tela_salvavel(&onze(), &[], inertes)),
        Ok(())
    );
}

#[test]
fn controle_enviavel_fora_do_observado_e_recusado() {
    for extra in [
        "<input type=\"text\" name=\"txtNovo\" value=\"\">",
        "<input type=\"hidden\" name=\"hdnIdUsuario\" value=\"1\">",
        "<select name=\"selOrdem\"><option value=\"1\">1</option></select>",
        "<textarea name=\"txaNota\"></textarea>",
        "<input type=\"radio\" name=\"rdoModo\" value=\"x\">",
        "<input type=\"submit\" name=\"sbmCancelar\" value=\"Cancelar\">",
        "<input type=\"image\" name=\"imgSalvar\" src=\"x.png\">",
        "<input type=\"checkbox\" name=\"chkOutra\">",
    ] {
        assert_eq!(
            conferir_controles_do_formulario(&tela_salvavel(&C::TODAS, &[], extra)),
            Err(ErroNivelDeDetalhe::Html(ErroHtml::valor(
                "campo_nao_previsto_no_formulario_do_nivel_de_detalhe"
            ))),
            "{extra}"
        );
    }
}

#[test]
fn marcadas_sai_na_ordem_do_catalogo() {
    let lido = nivel(&C::TODAS, &[C::Marcadores, C::Anotacao]);
    assert_eq!(lido.marcadas(), [C::Anotacao, C::Marcadores]);
    assert_eq!(
        C::pela_chave("ultima_movimentacao"),
        Some(C::UltimaMovimentacao)
    );
    assert_eq!(C::pela_chave("Tipo"), None);
}
