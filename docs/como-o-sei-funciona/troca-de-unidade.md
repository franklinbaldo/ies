---
type: SEIObservation
title: Troca da unidade da sessão pela tela Trocar Unidade
description: Onde o Controle publica a tela de troca, a lista de unidades com permissão, o selecionarUnidade() que cria selInfraUnidades, o POST, os dois 302 até o Controle da unidade nova (http:// no GO, infra_hash de 192 hexadecimais) e o que acontece com a URL da unidade anterior; cassettes UnB 4.0.9, UFRJ 4.1.5 e GO 5.0.3
---

# Troca de unidade

Observado nas cassettes públicas do Todos lidas pela estrutura
(04/11/2026): 9 cenários com o POST de `infra_trocar_unidade`, 22 trocas
gravadas (UnB 4.0.9: 8; UFRJ 4.1.5: 8; GO 5.0.3: 6), entre elas
`unidade-troca-e-volta-*` e `bloco-devolver-*`. PEN1 a PEN4 só gravaram o
GET da tela, sem POST. A gravação guarda status, `Content-Type`,
`Location` e corpo, não o corpo dos POST nem `Set-Cookie`. O fluxo de
comportamento é o do Todos (`src/todos/sei/troca.py`, medido na UnB).

## Onde a troca é publicada

O link da unidade na barra do sistema, `a#lnkInfraUnidade`, servido duas
vezes (barras móvel e padrão, mesmo texto e mesmo `onclick`) em toda tela
com a barra, inclusive o Controle de Processos:

```text
<a id="lnkInfraUnidade" href="#" onclick="window.location.href='controlador.php?acao=infra_trocar_unidade&infra_sistema=S&infra_unidade_atual=U&infra_hash=H';" …>SIGLA</a>
```

Igual nas três instalações. O texto do link é a sigla da unidade ativa; o
`infra_unidade_atual` do link é o id dela.

## A tela "Trocar Unidade"

O GET do link responde 200 com a barra (o mesmo `lnkInfraUnidade`) e:

- `form#frmInfraSelecaoUnidade method="post"`, `action` assinado
  `infra_trocar_unidade&acao_origem=infra_trocar_unidade`;
- filtros `txtInfraSiglaUnidade`, `txtInfraDescricaoUnidade` (vazios),
  `select#selInfraOrgaoUnidade` (`onchange="this.form.submit()"`) e o botão
  `sbmPesquisar` (Pesquisar);
- `table.infraTable` com a legenda `Lista de Unidades com Permissão (N registros):`
  (UnB 2, UFRJ 8 para `servidor1`, GO 5) e uma linha por unidade: rádio
  `chkInfraItem` com `value` = id, `title` = sigla e
  `onclick="infraSelecionarItens(this,'Infra'); selecionarUnidade(ID)"`, e as
  células Sigla, Descrição e Órgão. O rádio da unidade ativa vem `checked`;
- ocultos `hdnInfraTipoPagina`, `hdnInfraNroItens`, `hdnInfraItemId`,
  `hdnInfraItens` (os ids), `hdnInfraItensHash`, `hdnInfraItensSelecionados`
  (o id ativo), `hdnInfraSelecoes`, `hdnInfraPaginaSelecao`.

O JavaScript da tela, literal nas três:

```text
function selecionarUnidade(idUnidade){
var frm = document.getElementById('frmInfraSelecaoUnidade');
var input = document.createElement('input');
input.type = 'hidden';
input.name = 'selInfraUnidades';
input.value = idUnidade;
frm.appendChild(input);
frm.submit();
}
```

Clicar no rádio: `infraSelecionarItens` põe o id em
`hdnInfraItensSelecionados`, e `selecionarUnidade` cria o oculto
`selInfraUnidades` no fim do formulário e o envia por `submit()` (nenhum
botão participa). Sigla repetida na lista não foi vista.

## O POST e os dois redirecionamentos

As 22 trocas gravadas seguem a mesma sequência:

1. POST → **302** com `Location` para o Controle da unidade nova com aviso:
   `controlador.php?acao=procedimento_controlar&msg=Detectada troca de unidade A para B.&infra_sistema=S&infra_unidade_atual=NOVA&infra_hash=…`;
2. GET desse `Location` → **302** para o Controle sem o aviso:
   `controlador.php?acao=procedimento_controlar&infra_sistema=S&infra_unidade_atual=NOVA&infra_hash=…`;
3. GET → **200**: o Controle de Processos da unidade nova, com
   `lnkInfraUnidade` = sigla nova.

Formas do `Location`:

- absoluto `https://` no host na UnB e na UFRJ; **`http://` no mesmo host
  no GO** (os dois saltos). É a regra de autoridade compartilhada
  (`sufixo_da_mesma_origem`): o mesmo host em `http://` é a instalação;
- o `msg` vem com **espaços literais** no cabeçalho gravado; a URL que o
  Todos pediu em seguida (gravada) traz `%20`, a serialização do navegador;
- o `infra_hash` dos dois saltos tem **192** hexadecimais, não os 64 dos
  links das telas; os 128 finais são os mesmos nos dois saltos de uma
  troca. O Controle da unidade nova servido a seguir volta a publicar
  links de 64.

## Depois da troca

- **A unidade vale para a sessão inteira** (o cookie): qualquer
  requisição seguinte com o mesmo cookie, de qualquer agente, já é da
  unidade nova.
- **A URL do Controle da unidade anterior não volta à unidade anterior.**
  Na UnB (`unidade-troca-e-volta-unb`, passos 14 e 18), depois da troca para
  Unidade2, o GET da URL do Controle de Unidade1 respondeu 302 para o
  Controle de Unidade2 com `msg=Detectada troca de unidade Unidade1 para Unidade2.`
  A sessão não caiu. O Todos mediu que os links do menu da unidade anterior
  passam a ser recusados (302 para o login). Nos dois casos, uma URL guardada
  da unidade anterior não abre a tela que abria: os links assinados são da
  unidade (o hash cobre `infra_unidade_atual`).
- O GET repetido do `Location` com aviso respondeu 200 (UnB, passos 15 a
  22), sem novo 302. Só o primeiro, logo depois do POST, redirecionou.
- **Cookie.** As cassettes não gravam `Set-Cookie`. O fluxo do Todos não
  troca de cookie na troca, e nenhuma gravação mostra login depois dela.
  A mesma sessão segue válida na unidade nova (inferência das gravações:
  o Controle da unidade nova respondeu com o cookie de antes).

## Limites

Não observados: sigla repetida na lista, lista paginada (o maior
`servidor1` tem 8 unidades), resposta 200 ao POST, `Location` relativo e
caracteres fora do ASCII no `msg`. A troca nas PEN não foi gravada.
