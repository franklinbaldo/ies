---
type: CommandContract
title: Sobrestar e retomar processo na unidade da sessão
operation: processo.sobrestar
command: ies processo sobrestar NUP --motivo TEXTO [--confirmar]
effect: write-process-state
implementation: partial
---

# Sobrestar e retomar — #269

Comandos: `ies processo sobrestar NUP --motivo TEXTO [--confirmar]` e
`ies processo retomar NUP [--confirmar]`. Só Web, com a unidade da sessão
publicada pelo SEI. Nenhuma URL assinada, senha ou cookie é exibido.

## Entrada, prévia e confirmação implementadas offline

NUP usa o mesmo validador de `ies processo NUP`. `--motivo` é
obrigatório em sobrestar; branco ou vazio recusa com código de saída 2
(`ErroSobrestamento::MotivoVazio`) antes de qualquer arquivo ou rede.
O motivo aceito é preservado integralmente no formulário e na saída JSON,
incluindo espaços nas extremidades e quebras de linha.

Sem `--confirmar`: lê o processo pela mesma abertura de
[`processo concluir`](processo-concluir.md), sem link guardado da árvore,
pois ele não prova a unidade. Confere `lnkInfraUnidade` na tela de trabalho
e `aberto_em` do nó raiz. Sobrestar exige unidade ativa dentre as unidades
abertas e um único link assinado de `procedimento_sobrestar` na barra
do processo. Retomar exige `AbertoEm::Sobrestado` na unidade ativa
e um único `onclick="removerSobrestamentoProcesso();"` com URL da
tela do nó da mesma árvore. Confere a assinatura/ID, sem seguir 302
e sem executar JavaScript.

Resposta: `nup`, `id_procedimento`, `unidade`, `acao`, `motivo`
(quando sobrestar), `origem: web`, `confirmado: false`,
`alterado: false`, `requisicoes`. Não publica URL nem hash.

**Confirmação implementada no código:** `--confirmar` realiza a mesma
abertura da prévia, seguida de um GET da tela de sobrestar ou do nó,
de uma única escrita (POST com `rdoSobrestar=S` e `txaMotivo`,
ou GET da função de retirada) e uma nova leitura da árvore. Não há
retry de escrita, execução de JavaScript ou seguimento do 302. A
conferência exige `Sobrestado` na unidade para sobrestar, e
`AbertoEm::Unidades` com a unidade ativa para retomar. Se a
verificação falhar, sai com código 1 (escrita não confirmada). Um 302
isolado nunca prova a escrita. O JSON de sucesso só publica
`alterado:true` e `reconciliado_por_leitura:true` após
verificação. **Autenticação e escrita live ainda NOT_RUN**; o
código não foi exercitado em instalação de treinamento nesta PR.

## Requisições e evidências

| Caso | Requisições | Escritas |
| --- | --- | --- |
| Uso inválido (NUP/motivo) | 0 | 0 |
| Prévia a frio pelo Controle | 3 GET | 0 |
| Prévia com link de trabalho guardado | 2 GET | 0 |
| Confirmação sobrestar a frio | 5 GET + 1 POST = 6 | 1 POST |
| Confirmação retomar a frio | 6 GET = 6 | 1 GET de escrita |
| Recusa de situação/barra | só abertura | 0 |
| Recusa de tela | abertura + 1 GET | 0 |
| Falha após escrita | mesma contagem + leitura verificadora | 1, sem retry |

Cenários derivados de [Todos](https://github.com/franklinbaldo/todos/blob/main/src/todos/ciclo.py),
com cassettes de [UnB](../../cassettes/unb/processo-sobrestar-retomar-unb--cde2a847ebd8.md)
e [GO](../../cassettes/go/processo-sobrestar-retomar-go--4fb6c8fe18df.md).
RED da etapa inicial #279: 5 testes falhando. **RED da presente PR #284:
4 testes novos falhando**, antes do código (CI 42104973101).
As cinco instalações adicionais e o replay das cassettes do acervo
ainda não foram testados com esta implementação. Live: NOT_RUN,
0 requisições ao SEI e nenhum resíduo.

## Recusas tipadas (contrato de subcódigos)

| Erro (stderr) | Condição |
| --- | --- |
| `MotivoDoSobrestamentoVazio` | Não há motivo válido |
| `ErroSobrestamento::UnidadeNaoPublicada` | Unidade não observada |
| `ErroSobrestamento::SituacaoNaoLida` | Situação não observada |
| `ErroAcaoSobrestar::Barra(ErroBarraDoProcesso::BarraDoProcessoForaDaFormaObservada)` | Barra fora da forma |
| `ErroAcaoSobrestar::AcoesDeSobrestamentoRepetidasNaBarra` | Ações duplicadas |
| `ErroAcaoSobrestar::EnderecoDoSobrestamentoNaoReconhecido` | URL inválida |
| `ErroAcaoSobrestar::OutroProcesso` | ID diferente |
| `ErroAcaoSobrestar::TelaDoNoDaRetomadaNaoReconhecida` | URL da raiz não reconhecida |
| `ErroSobrestamento::JaSobrestado` | Já sobrestado |
| `ErroSobrestamento::NaoSobrestado` | Retomada não aplicável |
| `ErroSobrestamento::ForaDaUnidade` | Outra unidade |
| `ErroSobrestamento::SobrestadoEmOutraUnidade` | Outra unidade |
| `ErroSobrestamento::AcaoNaoOferecida` | Barra não oferece |
| `ErroSobrestamento::TelaRedirecionada` | GET da tela devolveu 302, antes do POST |
| `ErroSobrestamento::CharsetNaoSuportado` | Charset da tela não conhecido |
| `ErroTela::NaoInterpretada` | HTML da janela não interpretado |
| `ErroTela::FormularioAusente` | Formulário ausente |
| `ErroTela::OutroProcesso` | Identidade da janela diferente |
| `ErroTela::OpcaoNaoOferecida` | Rádio S ausente, repetido ou marcado de saída |
| `ErroTela::DestinoPreenchido` | Vínculo com outro processo preenchido |
| `ErroTela::CampoInesperado` | Campo extra, ausente, desabilitado ou repetido |
| `ErroEnderecoDeRetomar::EnderecoDeRetomarAusenteNaTela` | Função/variável não publicadas |
| `ErroEnderecoDeRetomar::EnderecoDeRetomarForaDaForma` | Forma da variável ou função não observada |
| `ErroEnderecoDeRetomar::EnderecosDeRetomarRepetidosNaTela` | Duas URLs de escrita |
| `ErroEnderecoDeRetomar::EnderecoDeRetomarNaoReconhecido` | URL assinada não reconhecida |
| `ErroEnderecoDeRetomar::OutroProcesso` | URL de outro id |
| `ErroSobrestamento::EscritaNaoAplicada` | Leitura confirma que o estado não mudou |
| `ErroSobrestamento::NaoConfirmado` | Leitura posterior falhou ou não decidiu (saída 1) |

[Decisão](../../docs/decisoes/processo-sobrestar.md) ·
[live](../../docs/validacao-live/2026-11-05-processo-sobrestar.md) ·
[PR #284](https://github.com/franklinbaldo/ies/pull/284) (continuação da #279).
