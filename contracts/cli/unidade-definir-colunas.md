---
type: CommandContract
title: Definir as colunas do Nível de detalhe
operation: unidade.definir_colunas
command: ies unidade colunas --definir todas|LISTA [--confirmar]
effect: write-user-preference
implementation: partial
---

# `ies unidade colunas --definir todas|LISTA [--confirmar]` (Refs #57)

Escreve, **uma vez e por pedido explícito**, quais colunas a
[visão detalhada](unidade-listar-processos-visao-detalhada.md) mostra: as
opções da tela **Nível de detalhe** (`procedimento_configurar_detalhe`), lidas
por [`ies unidade colunas`](unidade-colunas.md). O padrão do IES é o máximo
de colunas, aplicado por `--definir todas --confirmar` (decisão do
proprietário "B", 02/11/2026). Nenhuma leitura muda a configuração sozinha.

Segue as invariantes de escrita do [contrato da CLI](../cli.md): prévia,
confirmação e verificação; efeito incerto nunca se repete automaticamente;
saídas 0 sucesso, 1 falha, 2 uso inválido, 3 efeito incerto. Não há troca
de backend: só a Web, e a mesma sessão.

## Entrada

- `--definir todas`: todas as opções com `disponivel: true` **nesta**
  instalação (a UnB 4.0.9 publica 11, sem `prioridade`; a 5.x, 12).
- `--definir LISTA`: chaves estáveis separadas por vírgula, as mesmas de
  `ColunaDoDetalhe` em `colunas` e `colunas_ausentes`
  (`atribuicao,tipo,interessados,…`). É o **conjunto exato**: as listadas
  ficam marcadas, as demais desmarcadas. Espaços em volta de cada chave são
  ignorados; maiúsculas não são normalizadas.
- `--confirmar` (exige `--definir`): salva. Sem ele, só prévia.

Erros de uso, com **zero requisições** (antes de ler a configuração local),
saída 2 e `codigo: entrada_invalida`:

| Fato | Subcódigo |
| --- | --- |
| `--definir` vazio ou só espaços | `lista_de_colunas_vazia` |
| Item vazio entre vírgulas | `item_vazio_na_lista_de_colunas` |
| Chave fora das 12 (inclusive `todas` dentro de uma lista) | `coluna_desconhecida` (`coluna`: a chave digitada) |
| Chave repetida | `coluna_repetida_na_lista` (`coluna`) |
| `--confirmar` sem `--definir` | erro de uso do Clap |

## Comportamento

1. Lê a configuração pelo mesmo fluxo de `ies unidade colunas` (link
   `ancNivelDetalhe` da D; vinda da R, troca para D e **fica na D**), com
   as mesmas recusas.
2. Confere o formulário servido, **mesmo na prévia**: além da leitura,
   recusa controle enviável fora de `hdnInfraTipoPagina`, `sbmSalvar` e das
   caixas do catálogo (`campo_nao_previsto_no_formulario_do_nivel_de_detalhe`).
   Botões `button`/`reset` e controles `disabled` não são enviados e não
   são recusados.
3. Calcula o alvo. Coluna pedida que a tela não publica:
   `coluna_indisponivel_nesta_instalacao` (`coluna`), saída 1, sem POST de
   salvar. Alvo igual ao atual: sucesso com `alterado: false`, sem POST.
4. Deriva o POST pelo primitivo de formulário com a `const`
   `SALVAR_NIVEL_DE_DETALHE` da operação: `hdnInfraTipoPagina` como servido,
   `sbmSalvar` com o valor servido e uma caixa `on` por coluna do alvo, na
   ordem do documento. `sbmSalvar` ausente ou repetido é recusa do
   primitivo (`botao_de_envio_ausente_no_formulario`,
   `botoes_de_envio_repetidos_no_formulario`), antes de qualquer POST.
5. **Sem `--confirmar`**: publica antes, alvo, `marcar`, `desmarcar` e os
   nomes dos campos do POST que **seria** enviado. Zero escritas. Saída 0.
6. **Com `--confirmar`**: um POST de salvar (sem retry) e **um** GET da
   mesma URL do Nível de detalhe para verificar. Sucesso só se as marcas
   lidas forem exatamente o alvo.

A tela do Nível de detalhe é uma janela sem `lnkInfraUnidade` (ensaio da
PEN1): a unidade é a conferida na D que publicou o link, na mesma sessão
(`ConferenciaDaUnidade::NaTelaDeOrigemDoLink`); se a janela publicar
`lnkInfraUnidade`, ele também precisa ser a unidade solicitada.

## Resposta do POST e efeito incerto

Não há marcador de resultado observado: a resposta de sucesso é HTML 200
que fecha o modal. O primitivo aceita HTML 200 sem tela de login
(`ResultadoDoEnvio::ConferidoPorLeituraPosterior`), e a verificação por GET
é obrigatória. A resposta **não é a esperada** quando: falha de rede ou
tempo esgotado, 5xx, 429, 401/403, outro status, 302 (não seguido,
`redirecionamento_ao_salvar_nivel_de_detalhe`), MIME não HTML, charset,
tamanho ou tela de login. Então o IES **não repete** o POST: faz o mesmo GET
de verificação **uma vez** para reconciliar.

| Resposta do POST | Marcas lidas no GET | Saída | Resultado |
| --- | --- | --- | --- |
| Esperada | alvo | 0 | `alterado: true` |
| Esperada | as de antes | 1 | `escrita_nao_aplicada` / `configuracao_de_colunas_nao_aplicada` |
| Esperada | outras | 1 | `escrita_diferente_da_solicitada` / `marcas_do_nivel_de_detalhe_diferentes_do_alvo_apos_salvar` (`observadas`) |
| Não esperada | alvo | 0 | `alterado: true`, `reconciliado_por_leitura: true`, `falha_do_salvar` |
| Não esperada | as de antes | 1 | `configuracao_de_colunas_nao_aplicada`, com `falha_do_salvar` |
| Não esperada | outras | 3 | `efeito_incerto` / `efeito_incerto_na_configuracao_de_colunas` (`observadas`) |
| Qualquer | GET falhou ou tela irreconhecível | 3 | `efeito_incerto_na_configuracao_de_colunas` (`verificacao`: subcódigo) |

`falha_do_salvar` traz o subcódigo, e `status_http` e `mensagem_do_servidor`
quando o SEI respondeu com recusa HTTP ([contrato da CLI](../cli.md)). Um
servidor que processe o POST **depois** do GET de verificação não é
detectável: antes de repetir à mão, conferir com `ies unidade colunas`.
Falha **antes** do POST (arquivo de cookie, derivação) não é escrita:
saída 1, `fonte_invalida`.

## Efeito

`EfeitoAvaliado::AlteraPreferenciaDoUsuario { ColunasDaVisaoDetalhada }`:
preferência persistente do usuário, que vale na próxima leitura da D e não
volta sozinha (diferente de `AlteraEstadoDaSessao`). O IES não a desfaz:
para voltar, `--definir <antes> --confirmar` com o `antes` da saída. A
troca de visão vinda da R continua `PreferenciaDeVisualizacao`, não desfeita.

## Saída

```json
{"unidade": "TESTE", "origem": "web", "confirmado": false, "alterado": false,
 "antes": ["atribuicao", "tipo", "interessados"],
 "alvo": ["atribuicao", "anotacao", "tipo", "…"],
 "marcar": ["anotacao", "…"], "desmarcar": [],
 "indisponiveis": [],
 "envio": {"metodo": "POST", "acao": "procedimento_configurar_detalhe",
           "campos": ["hdnInfraTipoPagina", "sbmSalvar", "chkSinAtribuicao", "…"]},
 "depois": null, "reconciliado_por_leitura": false}
```

Listas na ordem do catálogo. `envio` é `null` sem POST (alvo igual ao
atual) e só nomes de campos, nunca valores, URL ou `infra_hash`. Com
`--confirmar` e sucesso: `confirmado: true`, `alterado: true`, `depois` com
as marcas lidas e, se reconciliado, `falha_do_salvar`. Falhas:
`{"codigo", "subcodigo", "erro", "retentavel": false}` mais `antes`, `alvo`,
`observadas` e `falha_do_salvar` quando houver. A saída humana mostra uma
linha por lista e termina com "Para aplicar, repita com --confirmar" na
prévia.

## Sequência e orçamento

Contando o GET do Controle. Sem rede para erro de uso.

| Caso | Sessão em D | Sessão em R |
| --- | --- | --- |
| Uso inválido | 0 | 0 |
| Prévia, alvo igual ao atual, coluna indisponível ou formulário recusado | 2 GET | 2 GET + 1 POST (D) |
| `--confirmar` que salva (inclusive reconciliado ou não aplicado) | 3 GET + 1 POST | 3 GET + 2 POST (D, salvar) |

O GET de verificação é sempre um só; não há segundo POST de salvar. Vinda
da R, a sessão fica na D (sem POST R de volta, decisão do proprietário,
02/11/2026); a recusa `visao_do_controle_nao_restaurada` e os campos
`restauracao`/`visao_deixada` da falha deixaram de existir.

## Fora do escopo

- `ies setup` pode, no futuro, oferecer `--definir todas` como passo
  opcional; não está ligado.
- Desfazer automaticamente ou "salvar e restaurar" a cada listagem
  (alternativa A, descartada na decisão).
- Opções fora do catálogo de 12.

## Evidência

Ensaio do proprietário na PEN1 5.0.4 (treinamento, 02/11/2026), registrado
no [contrato da leitura](unidade-colunas.md) e na
[validação live](../../docs/validacao-live/2026-11-02-leitura-controle.md):
formulário `frmAcessoExternoCancelar`, caixas `chkSin*` sem `value`, oculto
`hdnInfraTipoPagina`, `sbmSalvar` "Salvar", `hdnInfraPrefixoCookie` fora do
formulário; o POST de salvar envia só o oculto, o botão e as marcadas
`on`; resposta 200 sem redirect que fecha o modal; a D e o GET da tela
mostram a mudança em seguida. Ensaio deste comando: offline com telas
sintéticas de 12 e 11 opções; live **NOT_RUN**
([roteiro](../../docs/validacao-live/roteiro-definir-colunas.md),
[decisão](../../docs/decisoes/unidade-definir-colunas.md)).
