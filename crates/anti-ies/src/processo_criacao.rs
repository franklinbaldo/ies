//! Prévia de criação de processo no anti-IES, a partir de tipos do SQLite.
//! Layout 4.0 observado nos cards da `UnB`; valores de ensaio são hipóteses.
//! O POST criador permanece indisponível até uma fatia com prova de estado.
use crate::{Estado, Resposta, controlar, escapar, parametro_unico, protocolo_sintetico};
use iesei::criacao_processo::{self, TipoDeProcesso};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

const HASH: &str = protocolo_sintetico::INFRA_HASH;

/// Um cenário sem tipos observados não anuncia criação no menu.
pub(crate) fn acrescentar_menu(
    resposta: Resposta,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok {
        return resposta;
    }
    match criacao_processo::listar(banco, instalacao, cenario) {
        Ok(Some(_)) => {
            let mut resposta = resposta;
            if resposta.corpo.contains("</body>") {
                let link = format!(
                    "<a href='controlador.php?acao=procedimento_escolher_tipo&amp;infra_hash={HASH}'>Iniciar Processo</a>"
                );
                resposta.corpo = resposta
                    .corpo
                    .replacen("</body>", &format!("{link}</body>"), 1);
            }
            resposta
        }
        Ok(None) => resposta,
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

fn tela_de_tipos(tipos: &[TipoDeProcesso]) -> Resposta {
    let mut html = format!(
        "<html><body><form id='frmProcedimentoEscolherTipo' method='post' \
         action='controlador.php?acao=procedimento_escolher_tipo&amp;infra_hash={HASH}'>\
         <input type='hidden' name='hdnIdTipoProcedimento' value=''>\
         <input type='hidden' name='hdnFiltroTipoProcedimento' value='U'>\
         </form><button onclick=\"exibirTiposProcedimento('U')\">Todos</button>\
         <table id='tblTipoProcedimento'><tbody>"
    );
    for tipo in tipos {
        write!(
            html,
            "<tr><td><a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento={}&amp;infra_hash={HASH}'>{}</a></td></tr>",
            escapar(&tipo.id),
            escapar(&tipo.nome)
        )
        .expect("formatação String");
    }
    html.push_str("</tbody></table></body></html>");
    Resposta::html(html)
}

fn cadastro(tipo: &TipoDeProcesso) -> Resposta {
    Resposta::html(format!(
        "<html><body><form id='frmProcedimentoCadastro' method='post' \
         action='controlador.php?acao=procedimento_gerar&amp;infra_hash={HASH}'>\
         <input type='hidden' name='hdnIdTipoProcedimento' value='{}'>\
         <input type='hidden' name='hdnFlagProcedimentoCadastro' value='0'>\
         <input type='hidden' name='hdnAssuntos' value=''>\
         <input type='text' name='txtDescricao' maxlength='250' value=''>\
         <input type='radio' name='rdoNivelAcesso' value='0' checked>\
         <input type='radio' name='rdoNivelAcesso' value='1'>\
         <select name='selAssuntos'><option value='{}'>{}</option></select>\
         <select name='selHipoteseLegal'><option value='31'>Base Legal Teste</option></select>\
         </form></body></html>",
        escapar(&tipo.id),
        escapar(&tipo.id_assunto),
        escapar(&tipo.assunto),
    ))
}

/// Serve somente GETs de leitura da escolha e do cadastro.
///
/// O token do formulário é sintético; POST de criação não é implementado.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    match parametro_unico(alvo, "acao") {
        Some("procedimento_escolher_tipo") => {
            match criacao_processo::listar(banco, instalacao, cenario) {
                Ok(Some(tipos)) => tela_de_tipos(&tipos),
                Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        Some("procedimento_gerar") => {
            let Some(id) = parametro_unico(alvo, "id_tipo_procedimento") else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            match criacao_processo::consultar(banco, instalacao, cenario, id) {
                Ok(Some(tipo)) => cadastro(&tipo),
                Ok(None) => Resposta::nova(Estado::NaoEncontrado),
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        _ => Resposta::nova(Estado::NaoImplementado),
    }
}

/// Rota do Controle com menu derivado da tabela de tipos.
pub(crate) fn controle_com_menu(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    crate::blocos::acrescentar_menu(
        crate::prazo_processo::acrescentar_menu(
            acrescentar_menu(
                controlar(metodo, alvo, banco, instalacao, cenario),
                banco,
                instalacao,
                cenario,
            ),
            banco,
            instalacao,
            cenario,
        ),
        banco,
        instalacao,
        cenario,
    )
}
