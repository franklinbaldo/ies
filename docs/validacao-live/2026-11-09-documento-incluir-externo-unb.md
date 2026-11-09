---
type: SEIObservation
title: "UnB de treinamento: inclusão de PDF externo, escrita comprovada e três defeitos corrigidos"
description: "Ensaio de 09/11/2026 da #332: prévia PASS após duas correções, confirmação única com escrita comprovada por SHA-256, mas saída 3 por releitura defeituosa, corrigida offline com RED/GREEN"
---

# Documento incluir-externo — UnB de treinamento, 09/11/2026

**Issue:** [#332](https://github.com/franklinbaldo/ies/issues/332); fila de
ensaios [#583](https://github.com/franklinbaldo/ies/issues/583).
**Origem:** somente `https://sei.treinamento.unb.br/sei` (UnB SEI 4.0.9 de
**treinamento**), conta pública `aluno1` da
[tabela autorizada](../interfaces/credenciais-de-treinamento.md), unidade
exibida `Unidade1`. Arquivos de sessão e credenciais em pasta `0700` com
arquivos `0600`, fora do repositório. Nenhum cookie, URL assinada ou
`infra_hash` é publicado aqui.
**Execução:** binário release compilado localmente (Rust 1.99.1,
`--locked`) a partir dos commits listados abaixo, rede direta do sandbox.
**Autorização:** ensaio pedido pelo proprietário nesta sessão.

## Arquivo enviado

PDF sintético de 639 bytes, gerado para o ensaio (texto "Ensaio IES #332 -
documento externo - treinamento UnB 09/11/2026"), SHA-256
`1143eb8208320f677fdf841d77f6110071345f4b73da93be62575a337e7a5103`.
Processo escolhido entre os **Gerados** da Unidade1, já marcado como
visualizado (abri-lo não registra recebimento):
`23106.001906/2026-14` (Pessoal: Diárias de Viagens no Brasil).

## Orçamento

Declarado **25** requisições antes de começar; revisado para **35** antes
do `--confirmar`, porque duas prévias falhas de diagnóstico consumiram 7.
Total contado: **32** requisições publicadas pela CLI, mais a listagem de
Gerados, que não publica contagem. Nenhum retry; nenhuma requisição falha
foi repetida sem uma correção de código entre as tentativas.

## Sequência

| # | Commit | Comando | Resultado | Req. |
| --- | --- | --- | --- | --- |
| 1 | `528f9cb` | `ies --json login` | **PASS** `login_realizado` | 6 |
| 2 | `528f9cb` | `ies --json unidade gerados` | **PASS** 75 processos | n/p |
| 3 | `528f9cb` | `documento incluir-externo … --tipo Anexo --consultar-sei` | **FAIL** `link_externo_nao_conferido`, saída 1, sem upload | 3 |
| 4 | `4ef3561` | idem (prévia) | **FAIL** `tipo_externo_nao_oferecido`, saída 1, sem upload | 4 |
| 5 | `529b786` | idem (prévia) | **PASS** tipo `Anexo` → id `293`, limite PDF 209715200 bytes | 5 |
| 6 | `529b786` | `IES_ENSAIO_TREINAMENTO=332 … --confirmar` (uma vez, trava `O_EXCL`) | **saída 3**, `etapa: releitura_independente`, upload e cadastro tentados | 8 |
| 7 | `529b786` | `ies --json processo 23106.001906/2026-14` | **PASS**: árvore com `Anexo (0012272)` | 4 |
| 8 | `529b786` | `ies --json documento 0012272 --saida …` | **PASS**: PDF baixado, SHA-256 **idêntico** ao enviado | 2 |

O cadastro enviou data do dia, formato nato-digital (`N`) e nível de
acesso **público** (`0`), valores fixos da implementação.

## Diagnóstico dos três defeitos (offline, sem repetir requisições)

Cada causa foi achada no
[ZIP público da UnB](https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip)
(SHA-1 `21e0559015c56c403daf8933e5040504a7b83837`, conferido), não por
tentativa ao vivo.

1. **Link Externo com `id_procedimento` repetido** (passo 3). Cassette
   [documento-incluir-unb](../../cassettes/unb/documento-incluir-unb--cbaae321c0c1.md),
   interação 13: o link `documento_receber` publica
   `id_procedimento=51313` duas vezes. O conferidor comum exige uma.
   `documento criar` já tratava o mesmo caso localmente; o helper foi
   promovido a `arvore_processo::endereco` e passa a servir aos dois.
   Commit `4ef3561`.
2. **`&nbsp;` na opção vazia do `selSerie`** (passo 4). Cassette
   documento-mover-cancelar-unb, interações 16 e 17: a UnB serve
   `<option value="null">&nbsp;</option>`. O texto da opção passava pelo
   decodificador de **atributo**, que não conhece entidades Latin-1, e a
   tela inteira era recusada. Agora usa `entidades_html::decodificar_texto`.
   Commit `529b786`, RED com 2 testes novos.
3. **Rótulo do externo na releitura** (passo 6). A árvore publica o externo
   como `Anexo (0012272)`; a cassette mostra `Anexo (0011862)` nas
   interações 26–28. A releitura exigia a forma interna `Anexo 0012272` e
   devolvia efeito incerto mesmo com a escrita feita. O anti-IES também
   projetava a forma não observada e foi alinhado. Commit `c5504fc`: o E2E
   `bdd_332_03_binario_confirma_upload_cadastro_e_releitura_independente`
   reproduz a saída 3 sem a correção e passa com ela.

## Estado das provas

- **Escrita no SEI de treinamento: PASS**, comprovada por releitura
  independente (passos 7 e 8): documento novo `0012272`, ícone
  `documento_pdf`, não cancelado, conteúdo com SHA-256 idêntico ao enviado.
- **Saída da CLI no `--confirmar`: FAIL** no live (saída 3, falso efeito
  incerto). A correção (`c5504fc`) está validada só em
  IES ↔ anti-IES ↔ SQLite; **um segundo `--confirmar` ao vivo com o binário
  corrigido está NOT_RUN** e cria outro documento: depende de decisão do
  proprietário.
- **`--confirmar` público:** continua fechado. O ensaio usa o opt-in
  `IES_ENSAIO_TREINAMENTO=332` mais a origem exata da UnB de treinamento.

## Resíduo

- Documento externo **`0012272`** (Anexo, PDF de 639 bytes, público) no
  processo `23106.001906/2026-14` da Unidade1, UnB de treinamento. Não foi
  removido: cancelar é outra escrita e não fazia parte do roteiro.
- O upload temporário foi consumido pelo cadastro; não há temporário órfão
  conhecido.
- Os GETs de leitura podem ter registrado acesso no SEI; o processo já
  constava como visualizado.
