//! RED/GREEN do fluxo de `--definir` (contracts/cli/unidade-definir-colunas.md):
//! sequência de requisições pelo envio injetável, prévia sem escrita, um
//! único POST de salvar, verificação por um GET, reconciliação do efeito
//! incerto; vinda da R, a sessão fica na D.
use super::envio::EnvioDoControle;
use super::tests::{TelaD, TelaR, cookie, instalacao, tela_atual};
use super::*;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{ErroSubmissaoFormulario as S, RespostaDoPost, SubmissaoDerivada};
use crate::nivel_de_detalhe::tests_alvo::{HASH_DO_SALVAR, onze, tela_salvavel};
use crate::nivel_de_detalhe::tests_salvar::resposta_do_salvar;
use crate::nivel_de_detalhe::{AlvoDasColunas, ColunaDoDetalhe as C, ErroNivelDeDetalhe};
use crate::transporte::sessao::CookieDaSessao;
use crate::transporte::{FalhaTransporte, RespostaWeb, StatusHttpRecusado};
use std::io::Cursor;

/// Uma resposta roteirizada por requisição.
enum Passo {
    Html(i32, String),
    Falha(FalhaTransporte),
}

/// Registra `GET acao`, `POST visao=X` (troca) e `POST salvar` (com o corpo).
struct EnvioDoSalvar {
    roteiro: Vec<Passo>,
    registro: Vec<String>,
    corpos_do_salvar: Vec<String>,
}

impl EnvioDoSalvar {
    fn proxima(&mut self) -> Result<RespostaWeb<Cursor<Vec<u8>>>, FalhaTransporte> {
        assert!(!self.roteiro.is_empty(), "requisição além do orçamento");
        match self.roteiro.remove(0) {
            Passo::Falha(falha) => Err(falha),
            Passo::Html(status_code, html) => Ok(RespostaWeb {
                status_code,
                mime: "text/html; charset=iso-8859-1".to_owned(),
                content_length: None,
                localizacao: None,
                corpo: Cursor::new(
                    html.chars()
                        .map(|c| u8::try_from(u32::from(c)).expect("ISO-8859-1"))
                        .collect(),
                ),
            }),
        }
    }
}

impl EnvioDoControle for EnvioDoSalvar {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        let corpo = String::from_utf8(submissao.corpo().to_vec()).unwrap();
        if let Some(visao) = corpo
            .split('&')
            .find_map(|par| par.strip_prefix("hdnTipoVisualizacao="))
        {
            self.registro.push(format!("POST visao={visao}"));
        } else {
            assert!(
                submissao
                    .url()
                    .contains("acao=procedimento_configurar_detalhe")
            );
            assert!(submissao.url().contains(HASH_DO_SALVAR));
            assert!(
                submissao
                    .url()
                    .starts_with("https://sei.exemplo.gov.br/sei/controlador.php?")
            );
            self.registro.push("POST salvar".to_owned());
            self.corpos_do_salvar.push(corpo);
        }
        Ok(RespostaDoPost {
            resposta: self.proxima()?,
            localizacao: None,
        })
    }

    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        let acao = url
            .split(['?', '&'])
            .find_map(|p| p.strip_prefix("acao="))
            .unwrap_or("?");
        self.registro.push(format!("GET {acao}"));
        self.proxima()
    }
}

const ATUAL: [C; 3] = [C::Atribuicao, C::TipoProcesso, C::Interessados];
const GET_NIVEL: &str = "GET procedimento_configurar_detalhe";

fn d() -> TelaD {
    TelaD::nova(4, 0, 10, vec![1, 2, 3, 9])
}

fn r() -> TelaR {
    TelaR::nova((2, vec![1, 2]), (2, vec![3, 9]))
}

fn nivel(publicadas: &[C], marcadas: &[C]) -> Passo {
    Passo::Html(200, tela_salvavel(publicadas, marcadas, ""))
}

/// Resposta do salvar que gravou (PEN1 5.0.4): a tela com `'1'=='1'`.
fn fecha_modal() -> Passo {
    Passo::Html(200, resposta_do_salvar(&C::TODAS, &C::TODAS, "1"))
}

struct Rodada {
    resultado: Result<ColunasDefinidas, ErroDefinirColunas>,
    registro: Vec<String>,
    corpos: Vec<String>,
}

fn rodar(tela: String, alvo: &AlvoDasColunas, confirmar: bool, roteiro: Vec<Passo>) -> Rodada {
    let (_pasta, arquivo) = cookie();
    let mut envio = EnvioDoSalvar {
        roteiro,
        registro: Vec::new(),
        corpos_do_salvar: Vec::new(),
    };
    let resultado = definir_colunas(
        &instalacao(),
        &tela_atual(tela),
        Some("TESTE"),
        alvo,
        confirmar,
        &CookieDaSessao::ler(&arquivo),
        &mut envio,
    );
    assert!(envio.roteiro.is_empty(), "respostas não consumidas");
    Rodada {
        resultado,
        registro: envio.registro,
        corpos: envio.corpos_do_salvar,
    }
}

fn lista(colunas: &[C]) -> AlvoDasColunas {
    AlvoDasColunas::Lista(colunas.to_vec())
}

fn ok(rodada: &Rodada) -> &ColunasDefinidas {
    rodada.resultado.as_ref().unwrap()
}

fn erro(rodada: &Rodada) -> &ErroDefinirColunas {
    rodada.resultado.as_ref().unwrap_err()
}

#[test]
fn previa_na_d_le_planeja_e_nao_salva() {
    let rodada = rodar(
        d().html(),
        &AlvoDasColunas::Todas,
        false,
        vec![nivel(&C::TODAS, &ATUAL)],
    );
    assert_eq!(rodada.registro, [GET_NIVEL]);
    let feito = ok(&rodada);
    assert_eq!(feito.unidade, "TESTE");
    assert!(!feito.confirmado && !feito.alterado);
    assert_eq!(feito.plano.antes, ATUAL);
    assert_eq!(feito.plano.alvo, C::TODAS);
    let mut esperados = vec!["sbmSalvar".to_owned()];
    esperados.extend(C::TODAS.map(|c| c.campo().to_owned()));
    esperados.push("hdnInfraTipoPagina".to_owned());
    assert_eq!(feito.campos_do_envio.as_deref(), Some(&esperados[..]));
    assert_eq!(feito.depois, None);
    assert_eq!(feito.reconciliado_por, None);
}

#[test]
fn previa_na_r_troca_le_e_fica_na_d_sem_salvar() {
    let rodada = rodar(
        r().html(),
        &lista(&[C::Marcadores]),
        false,
        vec![Passo::Html(200, d().html()), nivel(&C::TODAS, &ATUAL)],
    );
    assert_eq!(rodada.registro, ["POST visao=D", GET_NIVEL]);
    let feito = ok(&rodada);
    assert_eq!(feito.plano.marcar(), [C::Marcadores]);
    assert_eq!(feito.plano.desmarcar(), ATUAL);
    assert_eq!(rodada.corpos.len(), 0);
}

#[test]
fn todas_confirmado_na_unb_salva_as_onze_e_confere_por_leitura() {
    let rodada = rodar(
        d().html(),
        &AlvoDasColunas::Todas,
        true,
        vec![
            nivel(&onze(), &ATUAL),
            fecha_modal(),
            nivel(&onze(), &onze()),
        ],
    );
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    let mut corpo = "sbmSalvar=Salvar".to_owned();
    for coluna in onze() {
        corpo.push('&');
        corpo.push_str(coluna.campo());
        corpo.push_str("=on");
    }
    corpo.push_str("&hdnInfraTipoPagina=2");
    assert_eq!(rodada.corpos, [corpo]);
    let feito = ok(&rodada);
    assert!(feito.confirmado && feito.alterado);
    assert_eq!(feito.plano.indisponiveis, [C::Prioridade]);
    assert_eq!(feito.depois.as_deref(), Some(&onze()[..]));
}

#[test]
fn lista_confirmada_na_r_salva_confere_e_fica_na_d() {
    let alvo = [C::Anotacao, C::Marcadores];
    let rodada = rodar(
        r().html(),
        &lista(&alvo),
        true,
        vec![
            Passo::Html(200, d().html()),
            nivel(&C::TODAS, &ATUAL),
            fecha_modal(),
            nivel(&C::TODAS, &alvo),
        ],
    );
    assert_eq!(
        rodada.registro,
        ["POST visao=D", GET_NIVEL, "POST salvar", GET_NIVEL]
    );
    assert_eq!(
        rodada.corpos,
        ["sbmSalvar=Salvar&chkSinAnotacao=on&chkSinMarcadores=on&hdnInfraTipoPagina=2"]
    );
    assert_eq!(ok(&rodada).depois.as_deref(), Some(&alvo[..]));
}

#[test]
fn alvo_igual_ao_atual_confirmado_nao_envia_post() {
    let rodada = rodar(
        d().html(),
        &lista(&ATUAL),
        true,
        vec![nivel(&C::TODAS, &ATUAL)],
    );
    assert_eq!(rodada.registro, [GET_NIVEL]);
    let feito = ok(&rodada);
    assert!(feito.confirmado && !feito.alterado);
    assert_eq!(feito.campos_do_envio, None);
}

#[test]
fn coluna_indisponivel_recusa_sem_salvar_e_fica_na_d() {
    let rodada = rodar(
        r().html(),
        &lista(&[C::Prioridade]),
        true,
        vec![Passo::Html(200, d().html()), nivel(&onze(), &ATUAL)],
    );
    assert_eq!(rodada.registro, ["POST visao=D", GET_NIVEL]);
    let falha = erro(&rodada);
    assert_eq!(
        falha,
        &ErroDefinirColunas::ColunaIndisponivelNestaInstalacao {
            coluna: C::Prioridade
        }
    );
}

#[test]
fn formulario_com_campo_nao_previsto_ou_sem_salvar_e_recusado_sem_post() {
    let extra = tela_salvavel(
        &C::TODAS,
        &ATUAL,
        "<input type=\"text\" name=\"txtNovo\" value=\"\">",
    );
    let sem_botao = tela_salvavel(&C::TODAS, &ATUAL, "").replacen("name=\"sbmSalvar\"", "", 1);
    for (html, esperado) in [
        (
            extra,
            ErroVisaoDoControle::NivelDeDetalhe(ErroNivelDeDetalhe::Html(ErroHtml::valor(
                "campo_nao_previsto_no_formulario_do_nivel_de_detalhe",
            ))),
        ),
        (
            sem_botao,
            ErroVisaoDoControle::Submissao(S::Html(ErroHtml::ausente(
                "botao_de_envio_ausente_no_formulario",
            ))),
        ),
    ] {
        for confirmar in [false, true] {
            let rodada = rodar(
                d().html(),
                &AlvoDasColunas::Todas,
                confirmar,
                vec![Passo::Html(200, html.clone())],
            );
            assert_eq!(rodada.registro, [GET_NIVEL]);
            assert_eq!(
                erro(&rodada),
                &ErroDefinirColunas::Leitura(esperado.clone())
            );
        }
    }
}

#[test]
fn verificacao_com_as_marcas_de_antes_e_nao_aplicada() {
    let rodada = rodar(
        d().html(),
        &AlvoDasColunas::Todas,
        true,
        vec![
            nivel(&C::TODAS, &ATUAL),
            fecha_modal(),
            nivel(&C::TODAS, &ATUAL),
        ],
    );
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    let falha = erro(&rodada);
    assert!(matches!(
        falha,
        ErroDefinirColunas::ConfiguracaoDeColunasNaoAplicada {
            falha_do_salvar: None,
            ..
        }
    ));
    assert_eq!(falha.codigo(), "escrita_nao_aplicada");
}

#[test]
fn verificacao_com_outras_marcas_apos_resposta_esperada_e_falha() {
    let rodada = rodar(
        d().html(),
        &AlvoDasColunas::Todas,
        true,
        vec![
            nivel(&C::TODAS, &ATUAL),
            fecha_modal(),
            nivel(&C::TODAS, &[C::Atribuicao]),
        ],
    );
    let falha = erro(&rodada);
    assert!(matches!(
        falha,
        ErroDefinirColunas::MarcasDiferentesDoAlvoAposSalvar { observadas, .. }
            if observadas == &[C::Atribuicao]
    ));
}

fn erro_500() -> FalhaTransporte {
    FalhaTransporte::StatusHttpDeErroDoServidor(StatusHttpRecusado {
        status: 500,
        mensagem_do_servidor: Some("Erro interno.".to_owned()),
    })
}

fn salvar_incerto(falha: Passo, verificacao: Passo) -> Rodada {
    rodar(
        d().html(),
        &AlvoDasColunas::Todas,
        true,
        vec![nivel(&C::TODAS, &ATUAL), falha, verificacao],
    )
}

#[test]
fn erro_5xx_no_salvar_reconcilia_por_leitura_sem_repetir_o_post() {
    let rodada = salvar_incerto(
        Passo::Html(500, "<html>Erro interno.</html>".to_owned()),
        nivel(&C::TODAS, &C::TODAS),
    );
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    let feito = ok(&rodada);
    assert!(feito.alterado);
    let por = feito.reconciliado_por.as_ref().unwrap();
    assert_eq!(por.status_http().map(|h| h.status), Some(500));
}

#[test]
fn tempo_esgotado_e_leitura_igual_a_antes_e_nao_aplicada() {
    let rodada = salvar_incerto(
        Passo::Falha(FalhaTransporte::Rede),
        nivel(&C::TODAS, &ATUAL),
    );
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    assert!(matches!(
        erro(&rodada),
        ErroDefinirColunas::ConfiguracaoDeColunasNaoAplicada {
            falha_do_salvar: Some(FalhaDoSalvar::Envio(S::Transporte(FalhaTransporte::Rede))),
            ..
        }
    ));
}

#[test]
fn tempo_esgotado_e_outras_marcas_e_nao_confirmado() {
    let rodada = salvar_incerto(
        Passo::Falha(FalhaTransporte::Rede),
        nivel(&C::TODAS, &[C::Atribuicao]),
    );
    let falha = erro(&rodada);
    assert!(matches!(
        falha,
        ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
            falha_do_salvar: Some(_),
            verificacao,
            ..
        } if **verificacao == VerificacaoDasColunas::Marcadas(vec![C::Atribuicao])
    ));
}

#[test]
fn verificacao_impossivel_apos_5xx_vinda_da_r_e_nao_confirmado_sem_post_de_volta() {
    let rodada = rodar(
        r().html(),
        &AlvoDasColunas::Todas,
        true,
        vec![
            Passo::Html(200, d().html()),
            nivel(&C::TODAS, &ATUAL),
            Passo::Falha(erro_500()),
            Passo::Falha(FalhaTransporte::Rede),
        ],
    );
    assert_eq!(
        rodada.registro,
        ["POST visao=D", GET_NIVEL, "POST salvar", GET_NIVEL]
    );
    let falha = erro(&rodada);
    assert!(matches!(
        falha,
        ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas { verificacao, .. }
            if matches!(**verificacao, VerificacaoDasColunas::LeituraFalhou(_))
    ));
}

#[test]
fn redirecionamento_ou_login_no_salvar_e_reconciliado_por_leitura() {
    let login =
        "<html><form id='frmLogin' action='login.php'><input name='txtUsuario'></form></html>"
            .to_owned();
    for (passo, _subcodigo) in [
        (Passo::Html(302, String::new()), "redirecionamento_302"),
        (Passo::Html(200, login), "tela_de_login_recebida_apos_envio"),
    ] {
        let rodada = salvar_incerto(passo, nivel(&C::TODAS, &C::TODAS));
        assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
        let _por = ok(&rodada).reconciliado_por.as_ref().unwrap();
    }
}

#[test]
fn resposta_esperada_e_verificacao_impossivel_e_nao_confirmado() {
    let rodada = salvar_incerto(fecha_modal(), Passo::Falha(FalhaTransporte::Rede));
    assert!(matches!(
        erro(&rodada),
        ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
            falha_do_salvar: None,
            verificacao,
            ..
        } if **verificacao == VerificacaoDasColunas::LeituraFalhou(
            ErroVisaoDoControle::Transporte(FalhaTransporte::Rede)
        )
    ));
}

#[test]
fn tela_devolvida_sem_sinal_e_leitura_de_antes_e_nao_aplicada_com_a_falha() {
    let rodada = salvar_incerto(
        Passo::Html(200, resposta_do_salvar(&C::TODAS, &ATUAL, "")),
        nivel(&C::TODAS, &ATUAL),
    );
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    let falha = erro(&rodada);
    assert!(matches!(
        falha,
        ErroDefinirColunas::ConfiguracaoDeColunasNaoAplicada {
            falha_do_salvar: Some(FalhaDoSalvar::NivelDevolvidoSemSinalDeGravacao),
            ..
        }
    ));
    assert_eq!(
        FalhaDoSalvar::NivelDevolvidoSemSinalDeGravacao.status_http(),
        None
    );
}

#[test]
fn resposta_200_nao_reconhecida_e_leitura_do_alvo_e_reconciliada() {
    let inventada =
        "<html><body><script>infraFecharJanelaModal();</script></body></html>".to_owned();
    let rodada = salvar_incerto(Passo::Html(200, inventada), nivel(&C::TODAS, &C::TODAS));
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    let por = ok(&rodada).reconciliado_por.as_ref().unwrap();
    assert_eq!(por, &FalhaDoSalvar::RespostaDoSalvarNaoReconhecida);
}

#[test]
fn sem_sinal_e_verificacao_impossivel_e_nao_confirmado_com_a_falha_do_salvar() {
    let rodada = salvar_incerto(
        Passo::Html(200, resposta_do_salvar(&C::TODAS, &ATUAL, "")),
        Passo::Falha(FalhaTransporte::Rede),
    );
    assert_eq!(rodada.registro, [GET_NIVEL, "POST salvar", GET_NIVEL]);
    assert!(matches!(
        erro(&rodada),
        ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
            falha_do_salvar: Some(FalhaDoSalvar::NivelDevolvidoSemSinalDeGravacao),
            verificacao,
            ..
        } if matches!(**verificacao, VerificacaoDasColunas::LeituraFalhou(_))
    ));
}
