---
type: CommandContract
title: Trocar a unidade da sessão
operation: unidade.trocar
command: ies unidade trocar SIGLA [--confirmar]
effect: write-session-state
implementation: partial
---

# `ies unidade trocar SIGLA [--confirmar]` (Refs #205)

Troca a unidade ativa da sessão pela tela **Trocar Unidade**
(`infra_trocar_unidade`), a mesma que o navegador abre pelo link da sigla
na barra. Observação em [troca de unidade](../../docs/como-o-sei-funciona/troca-de-unidade.md);
comportamento de referência no Todos (`src/todos/sei/troca.py`).

Segue as invariantes de escrita do [contrato da CLI](../cli.md): prévia,
confirmação e verificação; escrita não confirmada nunca se repete; saídas 0
sucesso, 1 falha, 2 uso inválido, 3 escrita não confirmada. Só a Web, na mesma
sessão; nenhum login.

**A troca vale para toda a sessão da conta.** A unidade é estado do cookie
no servidor: todo agente que usa a mesma sessão (os mesmos arquivos
`IES_*_FILE`) passa a ver a unidade nova no comando seguinte. O IES nunca
troca de unidade por conta própria, nenhum outro comando chama este, e
o IES não volta à unidade anterior: para voltar,
`ies unidade trocar <anterior> --confirmar`.

## Entrada

- `SIGLA`: a sigla como a tela a mostra (coluna Sigla, `title` do rádio),
  comparada byte a byte depois de tirar os espaços das pontas; maiúsculas
  não são normalizadas. Vazia ou só espaços: erro do clap, saída 2,
  antes de ler a configuração.
- `--confirmar`: envia a troca. Sem ele, só prévia.
- `--unidade U` (global): pré-condição da unidade **antes** da troca,
  conferida no Controle (`UnidadeExibidaDiferenteDaSolicitada`, saída
  1, sem a tela de troca).

## Comportamento

1. GET do Controle pela URL da sessão; a unidade ativa é o
   `lnkInfraUnidade` do Controle reconhecido (como `ies unidade atual`).
2. **Já na unidade** (`SIGLA` igual à ativa): sucesso, `alterado: false`,
   saída 0, sem a tela de troca e sem POST.
3. O link da troca é o `onclick` `window.location.href='controlador.php?acao=infra_trocar_unidade&…'`
   dos `lnkInfraUnidade` do Controle: todos iguais, uma `acao`, um
   `infra_hash` de 64 hexadecimais, sem `acao_ajax`, no controlador da
   instalação. GET da tela.
4. A tela é conferida: a mesma unidade na barra; `frmInfraSelecaoUnidade`;
   a legenda `Lista de Unidades com Permissão (N registros):` com N igual
   às linhas lidas; cada linha com um rádio `chkInfraItem` (id decimal,
   `title` = sigla da célula, `onclick` exatamente
   `infraSelecionarItens(this,'Infra'); selecionarUnidade(ID)`); o rádio
   marcado é a unidade ativa; o script define `selecionarUnidade` com o
   campo `selInfraUnidades` e `submit()`.
5. `SIGLA` entre as oferecidas: exatamente uma linha. Nenhuma:
   `UnidadeNaoOferecida` (com `oferecidas`); mais de uma:
   `SiglaDeUnidadeAmbigua` (com `candidatas`). Saída 1, sem POST.
6. Deriva o POST pelo primitivo de formulário (`TROCAR_UNIDADE`, envio
   `SubmitPorScriptComCampoCriado("selInfraUnidades")`): os ocultos como
   servidos, `hdnInfraItensSelecionados` = id, os filtros como servidos
   (vazios), `chkInfraItem` = id e, no fim, `selInfraUnidades` = id.
7. **Sem `--confirmar`**: publica a unidade ativa, a pedida, o id, as
   oferecidas e os nomes dos campos do POST que **seria** enviado. Saída 0.
8. **Com `--confirmar`**: obtém o lock do login (`<url-controle>.lock`)
   antes de ler a sessão e de qualquer requisição (#222: URL e cookie são
   lidos já sob o lock e o cookie lido é o que regrava a URL), envia **um** POST e segue os
   redirecionamentos observados, cada um **uma** vez:
   - POST → 302 para `procedimento_controlar` com `msg` e
     `infra_unidade_atual` = id escolhido;
   - GET desse `Location` → 302 para `procedimento_controlar` sem `msg`,
     mesmo id (ou 200 com o Controle, visto em GET repetido na UnB);
   - GET → 200: Controle reconhecido com `lnkInfraUnidade` = `SIGLA`.

   `Location` aceito: relativo `controlador.php?…`, ou absoluto `https://`
   ou `http://` no **host da instalação** (o GO responde `http://`), na
   pasta do controlador (`consulta_do_controlador`, a regra de origem do
   login). Os espaços literais do `msg` saem como `%20`, como o navegador
   os serializa; nenhum outro byte fora de 33–126 é aceito. A URL pedida é
   sempre `https://`. O `infra_hash` desses saltos tem 64 ou 192
   hexadecimais (192 observado nas três instalações).
9. Verificada a troca, grava a URL do último Controle como a URL da sessão
   (`IES_URL_CONTROLE_FILE`, 0600, `rename` atômico) e atualiza o marcador
   de geração com o mesmo cookie, sem novo login. Depois apaga **todos** os
   links assinados da instalação no índice local (ADR 0003) e colhe o
   Controle novo, já com a chave da unidade nova.

## Sessão e índice local

Os links assinados guardados levam `infra_unidade_atual` e são chaveados
por geração **e unidade**. Depois da troca nenhum é reaproveitado: a chave
da unidade muda com a URL gravada, e a troca apaga os da instalação (os da
unidade anterior respondem com outra tela: 302 para o Controle da unidade
nova, na UnB, ou para o login, medido pelo Todos). As identidades duráveis
(NUP ↔ id, número SEI ↔ ids) não dependem da unidade e ficam.

## Efeito

`EfeitoAvaliado::AlteraEstadoDaSessao { UnidadeAtual }`: muda o estado da
sessão no servidor para todos que a usam; não muda processo nem
preferência. Nada é restaurado.

## Saída

```json
{"antes": "PR-P", "pedida": "PR-A", "id_unidade": "110000938",
 "depois": null, "confirmado": false, "alterado": false,
 "escopo": "sessao_da_conta",
 "oferecidas": [{"sigla": "PR-A", "descricao": "Pró-Reitoria de Acesso", "id_unidade": "110000938"}, "…"],
 "envio": {"metodo": "POST", "acao": "infra_trocar_unidade",
           "campos": ["hdnInfraTipoPagina", "…", "selInfraUnidades"]},
 "links_descartados": null, "requisicoes": 2}
```

Com `--confirmar` e sucesso: `depois` = `SIGLA`, `confirmado: true`,
`alterado: true` e `links_descartados` (quantos links o índice apagou;
`null` com o índice desligado). Já na unidade: `alterado: false`,
`id_unidade`, `oferecidas` e `envio` `null`. `envio` traz só nomes, nunca
valores, URL ou `infra_hash`. A saída humana diz a unidade antes e
depois e repete que a troca vale para toda a sessão da conta.

## Falhas tipadas

Antes do POST (saída 1, nada mudou): as do GET do Controle (como
`ies unidade atual`, inclusive `SessaoEncerrada`),
`link_de_troca_de_unidade_ausente`, `links_de_troca_de_unidade_divergentes`,
`link_de_troca_de_unidade_fora_da_forma_observada`,
`tela_de_troca_de_unidade_nao_reconhecida`, `ListaDeUnidadesIncompleta`,
`linha_de_unidade_fora_da_forma_observada`,
`UnidadeMarcadaDiferenteDaAtiva`, `SelecaoDeUnidadePorScriptNaoPublicada`,
`UnidadeNaoOferecida`, `SiglaDeUnidadeAmbigua`, as do primitivo de
formulário e as do lock (`LoginEmAndamentoEmOutroProcesso`).

Depois do POST enviado, qualquer desvio do observado sai com **1** e
`Error: TrocaNaoFeita { erro: NaoConfirmado { etapa, falha }, requisicoes }`:
`etapa` é a `EtapaDaTroca` e `falha` a `FalhaDepoisDoEnvio` (`Envio`,
`RespostaSemRedirecionamento`, `LocalizacaoDeOutraUnidade`, `Status`,
`ControleNaoReconhecido`, `UnidadeDoControleDiferenteDaPedida`…). A URL da
sessão não é regravada e nenhum outro pedido sai: a unidade pode ter
mudado, e a URL guardada pode não abrir mais o Controle. Conferir com
`ies login --forcar` (sessão nova) antes de repetir. O índice é limpo
mesmo assim (nada da unidade anterior é reaproveitado).

Troca verificada mas URL da sessão não gravada:
`url_do_controle_da_unidade_nova_nao_gravada`, saída 1.

## Sequência e orçamento

| Caso | Requisições |
| --- | --- |
| Uso inválido | 0 |
| Já na unidade (com ou sem `--confirmar`) | 1 (Controle) |
| Prévia, ou recusa da tela | 2 (Controle, tela) |
| `--confirmar` observado | 5 (Controle, tela, POST, 302, Controle) |
| `--confirmar`, primeiro salto já 200 | 4 |

Validação live: **PASS** na [UFRJ 4.1.5](../../docs/validacao-live/2026-11-05-unidade-trocar.md) (05/11/2026; PR-P → PR-A → PR-P, prévia 2 e troca 5 requisições). UnB, GO e PEN: NOT_RUN.
Replay: [acervo](../../docs/validacao-replay/2026-11-05-unidade-trocar-acervo.md).
